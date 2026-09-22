# OnlyFans extractor investigation

API extraction remains disabled unless `YT_DLP_ONLYFANS_UNSAFE_API=1` is
explicitly set. Requests previously invalidated the browser session. No live
replay has been verified after these changes.

## Verified signing rules

Rules for revision `202609221225-841263d6aa` were recovered offline from the
user-supplied browser module `802313`. The user compared a successful browser
request with the recovered rules: full signature, SHA-1 digest, checksum on the
browser digest, prefix, suffix, and revision all matched. This establishes
agreement for that captured request, not future revisions or session acceptance.
`bin/lib/onlyfans-rules-candidate.json` retains the rules and source hash.

The offline checker reads the extractor without importing it:

```sh
python3 bin/onlyfans-signature-check.py
```

An unconfigured User-Agent is reported as `NOT CONFIGURED`; the signature checks
remain useful independently. Set `YT_DLP_ONLYFANS_USER_AGENT` to compare identity
as well. Do not store actual browser credentials in this repository.

## Browser identity

The opted-in extractor uses these environment variables from the same browser
session:

- `YT_DLP_ONLYFANS_USER_AGENT`: actual request User-Agent.
- `YT_DLP_ONLYFANS_X_BC`: optional override for the actual `x-bc` header.
  When unset, the extractor reads Brave local storage automatically.
- `YT_DLP_ONLYFANS_X_HASH`: actual `x-hash`, if present; otherwise leave unset.

Use private shell input rather than literal credential values in shell history.
These variables do not enable API access by themselves. A missing User-Agent, an unavailable browser key,
control characters in headers, or a missing `auth_id` cookie stop extraction.
The cookies must belong to the same browser session. Cookie import does not
import local storage.

The supplied page's module `916774` reuses `x-bc` in memory or local storage
(`bcTokenSha`); only an absent token triggers `/key/`. Module `290434` fetches
both `/key/` and `/hash/` without cross-origin credentials. The page stores the
hash in application state and throttles refresh calls for ten seconds.

The extractor reuses supplied or locally stored header values and makes neither CDN request.
This removes its previous fresh-key and CDN-cookie behavior. It does not
implement the page's hash refresh lifecycle: a captured hash may age, and long
profile downloads remain unverified. No browser header values are saved by the
extractor. Avoid verbose traffic logging when debugging authenticated requests.

Browser TLS impersonation is separate from these headers. Having curl_cffi
installed does not enable impersonation in this plugin; the existing `--porn`
alias requests it. A Chrome target is not proof of a match to the active Brave
session, even with the same User-Agent. Updating signing rules alone is not
proof that replay is safe.

## Offline regression checks

Run `scripts/tests/test_onlyfans.py` with the Python environment containing
yt-dlp. Tests use synthetic identities, reject socket connections, and cover
the guard, identity validation, CDN avoidance, signing, and offline checker.

Next: compare the updated extractor offline using the captured request. Any
live replay should be a separately agreed controlled test with the session
logout risk understood.

## Automatic Brave browser key

With `--cookies-from-browser brave`, an unset `YT_DLP_ONLYFANS_X_BC` triggers
`~/.local/bin/onlyfans-browser-key`. It uses yt-dlp's profile selection rules
(the newest Cookies database unless an explicit profile is supplied). Prefer
`--cookies-from-browser brave:Default` or the specific profile you use when
several profiles are active. No fallback to another profile is attempted.

The helper opens only a temporary copy of Local Storage/leveldb. It checks that
the source files stayed stable during copying and uses LevelDB's current view,
including deletions, instead of scanning old records for token-looking strings.
The temporary directory is private and is removed after reading. It reads only
the OnlyFans `bcTokenSha` entry from the copy and returns it through a captured
pipe; the extractor does not log it. The browser database is never opened for
writing or locked by the helper. Unflushed browser changes may not yet be on disk.

The helper uses an isolated uv script environment with Python 3.12 and pinned
`plyvel-ci==1.5.1`; first use can download these dependencies, but it never
contacts OnlyFans. It does not alter yt-dlp's Python environment. The User-Agent,
optional x-hash, and unsafe API opt-in are unchanged.

For a read-only check without displaying the key:

```sh
onlyfans-browser-key --check "$HOME/Library/Application Support/BraveSoftware/Brave-Browser/Default"
```

Storage regression tests run with `uv run scripts/tests/test_onlyfans_storage.py`.
