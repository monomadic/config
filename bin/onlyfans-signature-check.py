#!/usr/bin/env python3
"""Compare one captured request with the extractor's signing rules, offline."""

import ast
import argparse
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import sys
from urllib.parse import urlsplit


EXTRACTOR = (Path(__file__).resolve().parents[1] / 'config/yt-dlp/plugins'
             / 'faphouse/yt_dlp_plugins/extractor/onlyfans.py')


def load_constants(path=EXTRACTOR):
    # Read literals only: never import or initialize the network-capable extractor.
    tree = ast.parse(path.read_text())
    cls = next(node for node in tree.body
               if isinstance(node, ast.ClassDef) and node.name == 'OnlyFansIE')
    return {
        node.targets[0].id: ast.literal_eval(node.value)
        for node in cls.body if isinstance(node, ast.Assign)
        and isinstance(node.targets[0], ast.Name)
    }


def load_candidate(path, constants):
    """Translate data-only dynamic rules; never apply their header directives."""
    try:
        rules = json.loads(Path(path).read_text())
        if not isinstance(rules, dict):
            raise ValueError
        static = rules['static_param']
        indexes = rules['checksum_indexes']
        checksum = rules['checksum_constant']
        fmt = rules['format']
        if not isinstance(static, str) or not static or '\n' in static:
            raise ValueError
        if (not isinstance(indexes, list) or not indexes
                or any(type(i) is not int or not 0 <= i < 40 for i in indexes)
                or type(checksum) is not int):
            raise ValueError
        match = re.fullmatch(r'([^:{}]+):\{\}:\{:x\}:([^:{}]+)', fmt)
        if not match:
            raise ValueError
        for field in ('revision', 'user_agent'):
            if field in rules and (not isinstance(rules[field], str) or not rules[field].strip()):
                raise ValueError
    except (ValueError, KeyError, TypeError, OSError):
        raise ValueError('Candidate rules file is unreadable or has an unsupported schema.') from None
    return {
        **constants,
        '_STATIC_PARAM': static,
        '_SIGN_CHECKSUM_COEFS': tuple(indexes.count(i) for i in range(40)),
        '_SIGN_BASE_CHECKSUM': checksum,
        '_SIGN_PREFIX': match[1],
        '_SIGN_SUFFIX': match[2],
        '_REVISION': rules.get('revision', constants['_REVISION']),
        '_HEADERS': {**constants['_HEADERS'],
                     'User-Agent': rules.get('user_agent', constants['_HEADERS'].get('User-Agent', ''))},
        '_CANDIDATE_IDENTITY_FIELDS': tuple(
            label for field, label in (('revision', 'Revision'), ('user_agent', 'User-Agent'))
            if field in rules),
    }


def compare(constants, url, timestamp, user_id, signature, revision, user_agent):
    parsed = urlsplit(url)
    if (parsed.scheme != 'https' or parsed.netloc != 'onlyfans.com'
            or not parsed.path.startswith('/api2/v2/') or parsed.fragment):
        raise ValueError('Use the full HTTPS API Request URL, without a fragment.')
    if not re.fullmatch(r'[0-9]+', timestamp):
        raise ValueError('time must contain digits only; paste the time header value.')
    if not re.fullmatch(r'[0-9]+', user_id):
        raise ValueError('user-id must contain digits only; paste the user-id header value.')
    parts = signature.split(':')
    if (len(parts) != 4 or not re.fullmatch(r'[0-9a-f]{40}', parts[1])
            or not re.fullmatch(r'[0-9a-f]+', parts[2])):
        raise ValueError('sign must have four colon-separated parts with a SHA-1 digest.')
    # Preserve query ordering and escaping exactly as captured.
    path = parsed.path + ('?' + parsed.query if parsed.query else '')
    digest = hashlib.sha1(
        f'{constants["_STATIC_PARAM"]}\n{timestamp}\n{path}\n{int(user_id)}'.encode()
    ).hexdigest()
    checksum = abs(constants['_SIGN_BASE_CHECKSUM'] + sum(
        coef * ord(char) for coef, char in zip(constants['_SIGN_CHECKSUM_COEFS'], digest)))
    expected = f'{constants["_SIGN_PREFIX"]}:{digest}:{checksum:x}:{constants["_SIGN_SUFFIX"]}'
    # Also check the checksum rule against the captured digest independently.
    captured_checksum = abs(constants['_SIGN_BASE_CHECKSUM'] + sum(
        coef * ord(char) for coef, char in zip(constants['_SIGN_CHECKSUM_COEFS'], parts[1])))
    return {
        'Full signature': hmac.compare_digest(expected, signature),
        'SHA-1 digest': hmac.compare_digest(digest, parts[1]),
        'Checksum rule on browser digest': captured_checksum == int(parts[2], 16),
        'Signature prefix': constants['_SIGN_PREFIX'] == parts[0],
        'Signature suffix': constants['_SIGN_SUFFIX'] == parts[3],
        'Revision': constants['_REVISION'] == revision,
        'User-Agent': (constants['_HEADERS']['User-Agent'] == user_agent
                       if constants['_HEADERS'].get('User-Agent') else None),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rules', type=Path, help='Compare a local candidate rules JSON as well')
    args = parser.parse_args()
    if not sys.stdin.isatty():
        print('Run interactively; paste request values at the prompts.')
        return 2
    print('Offline check. No browser access, network requests, or saved values.')
    print('Use values from ONE successful post request in Brave Network > Headers.')
    print('Paste just each value, without its header name; input is visible.')
    print('Cookies, x-bc and x-hash are not needed for this signature check.\n')
    try:
        constants = load_constants()
        if os.environ.get('YT_DLP_ONLYFANS_USER_AGENT'):
            constants['_HEADERS']['User-Agent'] = os.environ['YT_DLP_ONLYFANS_USER_AGENT']
    except (ValueError, SyntaxError, OSError, StopIteration, KeyError):
        print('Unable to read signing constants from the local extractor file.')
        return 2
    try:
        candidate = load_candidate(args.rules, constants) if args.rules else None
    except ValueError as error:
        print(error)
        return 2
    try:
        values = [input(label + ': ').strip() for label in (
            'Request URL', 'time', 'user-id', 'sign', 'x-of-rev', 'user-agent')]
        results = compare(constants, *values)
    except (KeyboardInterrupt, EOFError):
        print('\nCancelled; no values saved.')
        return 2
    except ValueError as error:
        print(f'Unable to check: {error}')
        return 2
    except KeyError:
        print('Unable to check: the local extractor is missing a required signing constant.')
        return 2
    print('\nResults (safe to share):')
    for label, matches in results.items():
        print(f'{label}: {"NOT CONFIGURED" if matches is None else "MATCH" if matches else "MISMATCH"}')
    if candidate is not None:
        candidate_results = compare(candidate, *values)
        print('\nCandidate signing rules (not applied to extractor):')
        for label, matches in candidate_results.items():
            if label not in ('Revision', 'User-Agent') or label in candidate['_CANDIDATE_IDENTITY_FIELDS']:
                print(f'{label}: {"NOT CONFIGURED" if matches is None else "MATCH" if matches else "MISMATCH"}')
        print('Identity comparisons use supplied candidate values, not a live browser check.')
    print('\nThis does not validate cookies, x-bc, x-hash, or session acceptance.')
    print('Even a full match does not establish that replay is safe. Do not retry the API yet.')
    return 0 if all(value for value in results.values() if value is not None) else 1


if __name__ == '__main__':
    sys.exit(main())
