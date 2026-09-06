#!/usr/bin/env python3
"""Compare one captured request with the extractor's signing rules, offline."""

import ast
import getpass
import hashlib
import hmac
from pathlib import Path
import re
import sys
from urllib.parse import urlsplit
import warnings


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


def compare(constants, url, timestamp, user_id, signature, revision, user_agent):
    parsed = urlsplit(url)
    if (parsed.scheme != 'https' or parsed.netloc != 'onlyfans.com'
            or not parsed.path.startswith('/api2/v2/') or parsed.fragment):
        raise ValueError('Use the full HTTPS API Request URL, without a fragment.')
    if not re.fullmatch(r'[0-9]+', timestamp) or not re.fullmatch(r'[0-9]+', user_id):
        raise ValueError('time and user-id must contain digits only.')
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
        'User-Agent': constants['_HEADERS']['User-Agent'] == user_agent,
    }


def main():
    if len(sys.argv) != 1 or not sys.stdin.isatty():
        print('Run interactively without arguments; paste values only at the hidden prompts.')
        return 2
    print('Offline check. No browser access, network requests, or saved values.')
    print('Use values from ONE successful post request in Brave Network > Headers.')
    print('Paste just each value, without its header name; input is hidden.')
    print('Cookies, x-bc and x-hash are not needed for this signature check.\n')
    try:
        constants = load_constants()
        with warnings.catch_warnings():
            # Refuse getpass fallback rather than echoing sensitive input.
            warnings.simplefilter('error', getpass.GetPassWarning)
            values = [getpass.getpass(label + ': ').strip() for label in (
                'Request URL', 'time', 'user-id', 'sign', 'x-of-rev', 'user-agent')]
        results = compare(constants, *values)
    except (KeyboardInterrupt, EOFError, getpass.GetPassWarning):
        print('\nCancelled; no values saved.')
        return 2
    except (ValueError, OSError, StopIteration, KeyError):
        print('Unable to check: verify the six values and the extractor file. No values printed.')
        return 2
    print('\nResults (safe to share):')
    for label, matches in results.items():
        print(f'{label}: {"MATCH" if matches else "MISMATCH"}')
    print('\nThis does not validate cookies, x-bc, x-hash, or session acceptance.')
    print('Even a full match does not establish that replay is safe. Do not retry the API yet.')
    return 0 if all(results.values()) else 1


if __name__ == '__main__':
    sys.exit(main())
