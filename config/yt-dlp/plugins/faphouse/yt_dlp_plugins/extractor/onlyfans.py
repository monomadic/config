import hashlib
import os
from pathlib import Path
import plistlib
import re
import sys
import subprocess
import time
from urllib.parse import urlencode, urljoin, urlparse

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    float_or_none,
    int_or_none,
    traverse_obj,
    try_get,
    url_or_none,
)


class OnlyFansIE(InfoExtractor):
    IE_NAME = 'onlyfans'
    _VALID_URL = r'https?://(?:www\.)?onlyfans\.com/(?P<id>[A-Za-z0-9_.-]+)(?:/(?:posts?|videos?)/(?P<post_id>\d+))?'
    _API_BASE = 'https://onlyfans.com/api2/v2/'
    _APP_TOKEN = '33d57ade8c02dbc5a333db99ff9ae26a'
    # Recovered from browser module 802313; matched a successful request offline.
    _REVISION = '202609221225-841263d6aa'
    _STATIC_PARAM = 'hrydcSu7lgHNmQl7QUpyrTGfJcvmFmLU'
    _SIGN_PREFIX = '65571'
    _SIGN_SUFFIX = '6ab273d0'
    _SIGN_BASE_CHECKSUM = -233
    _SIGN_CHECKSUM_COEFS = (
        1, 2, 1, 1, 3, 1, 0, 2, 1, 1,
        3, 0, 1, 1, 0, 0, 2, 0, 1, 0,
        1, 0, 0, 0, 1, 0, 0, 0, 1, 1,
        0, 2, 1, 0, 1, 2, 0, 1, 0, 0,
    )
    _HEADERS = {
        'Accept': 'application/json, text/plain, */*',
        'Content-Type': 'application/json',
    }
    _AUTH_HINT = (
        'OnlyFans rejected the request authentication. This does not establish that '
        'the cookies are invalid: the signing rules, browser identity, or session '
        'headers may not match. Stop replaying requests and compare against a '
        'successful browser request before retrying.'
    )
    _UNSAFE_OPT_IN = 'YT_DLP_ONLYFANS_UNSAFE_API'

    def _real_initialize(self):
        self._check_api_opt_in()
        # Reuse identity from a successful browser request. Cookie extraction does
        # not import localStorage's bcTokenSha. Never mint a replacement via /key/.
        self._browser_user_agent = self._identity_value('USER_AGENT') or self._installed_browser_user_agent()
        self._bc_token = self._identity_value('X_BC') or self._stored_browser_key()
        self._hash = self._identity_value('X_HASH')
        cookies = self._get_cookies('https://onlyfans.com/')
        self._auth_user_id = int_or_none(try_get(cookies, lambda x: x['auth_id'].value))
        if not self._auth_user_id:
            raise ExtractorError('OnlyFans requires an auth_id cookie from the matching browser session.', expected=True)

    def _installed_browser_user_agent(self):
        spec = self.get_param('cookiesfrombrowser')
        hint = 'Set YT_DLP_ONLYFANS_USER_AGENT to the actual browser request header.'
        if sys.platform != 'darwin' or not spec or spec[0] != 'brave':
            raise ExtractorError('Automatic User-Agent supports Brave on macOS. ' + hint, expected=True)
        # Brave's macOS bundle version starts with the Chromium major version.
        # Reconstruct the standard reduced UA; custom UA overrides need the env var.
        candidates = [Path('/Applications/Brave Browser.app'),
                      Path.home() / 'Applications/Brave Browser.app']
        versions = set()
        for app in candidates:
            info = app / 'Contents/Info.plist'
            if not info.exists():
                continue
            try:
                data = plistlib.loads(info.read_bytes())
                version = data.get('CFBundleShortVersionString', '')
                if data.get('CFBundleIdentifier') != 'com.brave.Browser' or not re.fullmatch(r'[0-9]{3,}\.[0-9]+\.[0-9]+\.[0-9]+', version):
                    raise ValueError('Unrecognized Brave version')
                versions.add(version.split('.')[0])
            except (OSError, ValueError, TypeError, plistlib.InvalidFileException):
                raise ExtractorError('Could not read the installed Brave version. ' + hint, expected=True) from None
        if len(versions) != 1:
            raise ExtractorError('Installed Brave version is missing or ambiguous. ' + hint, expected=True)
        major = versions.pop()
        return ('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) '
                'AppleWebKit/537.36 (KHTML, like Gecko) '
                f'Chrome/{major}.0.0.0 Safari/537.36')

    def _stored_browser_key(self):
        # Use yt-dlp's own profile selection rules, including newest Cookies DB.
        from yt_dlp.cookies import (
            YDLLogger, _find_files, _get_chromium_based_browser_settings,
            _is_path, _newest, _parse_browser_specification,
        )
        spec = self.get_param('cookiesfrombrowser')
        if not spec:
            raise ExtractorError('Automatic x-bc requires --cookies-from-browser brave; '
                                 'otherwise set YT_DLP_ONLYFANS_X_BC.', expected=True)
        browser, profile, _, _ = _parse_browser_specification(*spec)
        if browser != 'brave':
            raise ExtractorError('Automatic x-bc currently supports Brave only; '
                                 'set YT_DLP_ONLYFANS_X_BC for another browser.', expected=True)
        root = _get_chromium_based_browser_settings(browser)['browser_dir']
        if profile is not None:
            root = profile if _is_path(profile) else os.path.join(root, profile)
        cookie_db = _newest(_find_files(root, 'Cookies', YDLLogger(self._downloader)))
        if not cookie_db:
            raise ExtractorError('Could not locate the Brave cookie profile.', expected=True)
        profile_dir = Path(cookie_db).parent
        if profile_dir.name == 'Network':
            profile_dir = profile_dir.parent
        helper = Path.home() / '.local/bin/onlyfans-browser-key'
        try:
            result = subprocess.run([str(helper), str(profile_dir)], capture_output=True,
                                    text=True, timeout=60, check=True)
            value = result.stdout.rstrip('\n')
            if not value.strip() or any(ord(c) < 32 or ord(c) == 127 for c in value):
                raise ValueError('Invalid browser key')
            return value
        except (OSError, subprocess.SubprocessError, ValueError):
            raise ExtractorError('Could not read Brave local storage. Ensure onlyfans-browser-key '
                                 'and uv are installed, or set YT_DLP_ONLYFANS_X_BC.', expected=True) from None

    def _identity_value(self, name, required=False):
        variable = f'YT_DLP_ONLYFANS_{name}'
        value = os.environ.get(variable, '')
        if (required and not value.strip()) or any(ord(char) < 32 or ord(char) == 127 for char in value):
            raise ExtractorError(
                f'Set {variable} to the corresponding header from a successful browser request '
                '(a single nonempty line for required headers).', expected=True)
        return value

    def _signed_headers(self, api_path, referer, user_id=None):
        user_id = user_id if user_id is not None else self._auth_user_id
        timestamp = int(time.time() * 1000)
        sha1 = hashlib.sha1(
            f'{self._STATIC_PARAM}\n{timestamp}\n{api_path}\n{user_id or 0}'.encode()
        ).hexdigest()
        checksum = abs(self._SIGN_BASE_CHECKSUM + sum(
            coef * ord(char)
            for coef, char in zip(self._SIGN_CHECKSUM_COEFS, sha1)
        ))
        headers = {
            **self._HEADERS,
            'User-Agent': self._browser_user_agent,
            'Referer': referer,
            'app-token': self._APP_TOKEN,
            'time': str(timestamp),
            'sign': f'{self._SIGN_PREFIX}:{sha1}:{checksum:x}:{self._SIGN_SUFFIX}',
            'x-bc': self._bc_token,
            'x-of-rev': self._REVISION,
        }
        if self._hash:
            headers['x-hash'] = self._hash
        if user_id:
            headers['user-id'] = str(user_id)
        return headers

    def _api_path(self, path, query=None):
        if query:
            return f'{path}?{urlencode(query)}'
        return path

    def _check_api_opt_in(self):
        if os.environ.get(self._UNSAFE_OPT_IN) != '1':
            raise ExtractorError(
                'OnlyFans API extraction is disabled because the current signed request '
                'replay can invalidate the browser session. Set '
                f'{self._UNSAFE_OPT_IN}=1 only when intentionally debugging this extractor.',
                expected=True)

    def _download_api_json(self, path, video_id, note, referer, query=None, user_id=None):
        self._check_api_opt_in()
        api_path = self._api_path(path, query)
        api_url = urljoin(self._API_BASE, api_path.lstrip('/'))
        parsed_api_url = urlparse(api_url)
        signed_path = parsed_api_url.path
        if parsed_api_url.query:
            signed_path = f'{signed_path}?{parsed_api_url.query}'
        response = self._download_json(
            api_url, video_id,
            note=note, headers=self._signed_headers(signed_path, referer, user_id),
            expected_status=(400, 401, 403, 404))

        error = response.get('error') if isinstance(response, dict) else None
        if error:
            message = error.get('message') or 'OnlyFans API request failed'
            code = error.get('code')
            if code in (101, 301, 400, 401) or any(term in message.lower() for term in ('session', 'wrong user')):
                message = f'{message} ({self._AUTH_HINT})'
            raise ExtractorError(message, expected=True)
        return response

    def _extract_formats_from_media(self, media, post_id):
        formats = []
        media_id = str(media.get('id') or post_id)

        source = url_or_none(
            media.get('source')
            or traverse_obj(media, ('files', 'source', 'url', {str}, any)))
        if source:
            formats.append({
                'url': source,
                'format_id': f'{media_id}-source',
                'quality': 10,
            })

        for quality, item in enumerate(traverse_obj(media, ('files', 'drm', ..., {dict})) or []):
            url = url_or_none(item.get('url'))
            if url:
                formats.append({
                    'url': url,
                    'format_id': f'{media_id}-drm-{quality}',
                    'quality': -10 + quality,
                })

        for label, obj in (traverse_obj(media, ('files', {dict})) or {}).items():
            if not isinstance(obj, dict):
                continue
            url = url_or_none(obj.get('url') or obj.get('src'))
            if not url or url == source:
                continue
            formats.append({
                'url': url,
                'format_id': f'{media_id}-{label}',
            })

        return formats

    def _entries_from_posts(self, posts, username):
        for post in posts:
            post_id = str(post.get('id') or '')
            if not post_id:
                continue
            post_url = f'https://onlyfans.com/{post_id}/{username}'
            title = (
                post.get('rawText')
                or post.get('text')
                or f'{username} post {post_id}')
            timestamp = int_or_none(post.get('postedAtPrecise')) or int_or_none(post.get('postedAt'))
            media_items = traverse_obj(post, ('media', ..., {dict})) or []

            for index, media in enumerate(media_items, 1):
                formats = self._extract_formats_from_media(media, post_id)
                if not formats:
                    continue
                media_id = str(media.get('id') or f'{post_id}-{index}')
                yield {
                    'id': media_id,
                    'display_id': post_id,
                    'title': title,
                    'webpage_url': post_url,
                    'description': post.get('text'),
                    'timestamp': timestamp,
                    'duration': float_or_none(media.get('duration')),
                    'thumbnail': url_or_none(media.get('preview') or media.get('thumb')),
                    'uploader': username,
                    'channel': username,
                    'age_limit': 18,
                    'formats': formats,
                }

    def _extract_profile(self, url, username):
        user = self._download_api_json(
            f'/users/{username}', username, 'Downloading profile metadata', url)
        user_id = int_or_none(user.get('id'))
        if not user_id:
            raise ExtractorError('OnlyFans did not return a usable profile id', expected=True)

        entries = []
        offset = 0
        limit = 50
        while True:
            query = {
                'limit': limit,
                'offset': offset,
                'order': 'publish_date_desc',
                'skip_users': 'all',
                'format': 'infinite',
            }
            posts = self._download_api_json(
                f'/users/{user_id}/posts', username,
                f'Downloading posts page {offset // limit + 1}', url,
                query=query)
            post_list = posts.get('list') if isinstance(posts, dict) else None
            if not post_list:
                break
            entries.extend(self._entries_from_posts(post_list, username))
            if not posts.get('hasMore'):
                break
            offset = int_or_none(posts.get('nextOffset')) or offset + limit

        if not entries:
            raise ExtractorError(
                'No downloadable media found. The session may not be logged in, subscribed, or allowed to view this profile.',
                expected=True)

        return self.playlist_result(
            entries, playlist_id=str(user_id),
            playlist_title=user.get('name') or user.get('username') or username,
            playlist_description=user.get('about'))

    def _extract_post(self, url, username, post_id):
        post = self._download_api_json(
            f'/posts/{post_id}', post_id, 'Downloading post metadata',
            url, query={'skip_users': 'all'})
        entries = list(self._entries_from_posts([post], username))
        if not entries:
            raise ExtractorError('No downloadable media found in this OnlyFans post', expected=True)
        return self.playlist_result(entries, playlist_id=post_id, playlist_title=try_get(post, lambda x: x['text']))

    def _real_extract(self, url):
        username, post_id = self._match_valid_url(url).group('id', 'post_id')
        path_parts = [part for part in urlparse(url).path.split('/') if part]
        if not post_id and len(path_parts) >= 2 and path_parts[0].isdigit():
            post_id, username = path_parts[:2]
        elif not post_id and len(path_parts) >= 2 and path_parts[1].isdigit():
            post_id = path_parts[1]

        if post_id:
            return self._extract_post(url, username, post_id)
        return self._extract_profile(url, username)
