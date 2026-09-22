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

Before accessing cookies, the opted-in extractor requires these environment
variables, copied from the same successful browser request:

- `YT_DLP_ONLYFANS_USER_AGENT`: actual request User-Agent.
- `YT_DLP_ONLYFANS_X_BC`: actual `x-bc` header.
- `YT_DLP_ONLYFANS_X_HASH`: actual `x-hash`, if present; otherwise leave unset.

Use private shell input rather than literal credential values in shell history.
These variables do not enable API access by themselves. Missing identity inputs,
control characters in headers, or a missing `auth_id` cookie stop extraction.
The cookies must belong to the same browser session. Cookie import does not
import local storage.

The supplied page's module `916774` reuses `x-bc` in memory or local storage
(`bcTokenSha`); only an absent token triggers `/key/`. Module `290434` fetches
both `/key/` and `/hash/` without cross-origin credentials. The page stores the
hash in application state and throttles refresh calls for ten seconds.

The extractor now reuses supplied header values and makes neither CDN request.
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
