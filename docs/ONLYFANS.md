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

The offline checker does not inspect the browser installation. Without a User-Agent
override it reports `NOT CONFIGURED`; the signature checks
remain useful independently. Set `YT_DLP_ONLYFANS_USER_AGENT` to compare identity
as well. Do not store actual browser credentials in this repository.

## Browser identity

The opted-in extractor uses these environment variables from the same browser
session:

- `YT_DLP_ONLYFANS_USER_AGENT`: optional override for the actual request
  User-Agent. Otherwise it is derived from the installed Brave version on macOS.
- `YT_DLP_ONLYFANS_X_BC`: optional override for the actual `x-bc` header.
  When unset, the extractor reads Brave local storage automatically.
- `YT_DLP_ONLYFANS_X_HASH`: actual `x-hash`, if present; otherwise leave unset.

Use private shell input rather than literal credential values in shell history.
These variables do not enable API access by themselves. An unavailable browser identity or browser key,
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
contacts OnlyFans. It does not alter yt-dlp's Python environment. The optional x-hash and unsafe API opt-in are unchanged.

For a read-only check without displaying the key:

```sh
onlyfans-browser-key --check "$HOME/Library/Application Support/BraveSoftware/Brave-Browser/Default"
```

Storage regression tests run with `uv run scripts/tests/test_onlyfans_storage.py`.

## Automatic Brave User-Agent

On macOS, with Brave selected for cookies, the extractor reads the stable Brave
application's Info.plist under /Applications or ~/Applications. It derives the
Chromium major version from CFBundleShortVersionString and builds the standard
[reduced macOS User-Agent](https://www.chromium.org/updates/ua-reduction/).
For the installed Brave 153.1.95.104 this exactly matches the user's supplied
Chrome/153.0.0.0 request header. No browser launch or network request is needed.

This reconstructs the default header; it does not read navigator.userAgent from
a running tab. A custom User-Agent, nonstandard install location, or an older
browser still running after an on-disk update needs the explicit environment
override. Conflicting major versions in the two install locations cause an error
rather than choosing one. The TLS impersonation target remains separate.

## Metadata and profile feeds

Titles and descriptions are cleaned of HTML and entities. The first line of the
clean caption is used as the title; the full clean caption is the description.
Creator display names populate channel/uploader, the account handle populates
uploader_id, and the numeric account ID populates channel_id. These fields are
also attached to playlist results. A post containing one downloadable media item
returns that media directly, avoiding playlist-level metadata loss in callers.
Single-post extraction may fetch creator metadata when the post author is partial.

Named users in releaseForms populate yt-dlp's cast field (the existing config maps
cast into actors). Mentions and linkedUsers are not cast: the supplied browser
code also uses linkedUsers for promotional posts. Missing or ID-only participant
records do not establish names; the extractor leaves cast empty in that case.

Profile /videos URLs select /users/<id>/posts/videos and filter nonvideo media.
Bare profiles select the general posts feed. Pagination is lazy and uses
beforePublishTime with six decimal places, following the supplied browser's
fetchUserPosts code. Decimal publication markers are preferred, with
postedAtPrecise as fallback. Repeated post IDs are suppressed and a missing or
nonadvancing cursor fails instead of looping. Unsupported sections fail explicitly.
These mappings and pagination are covered by synthetic response tests; no live
profile request was made to validate this change.
