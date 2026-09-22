import hashlib
from decimal import Decimal, InvalidOperation
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
    clean_html,
    unified_timestamp,
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

    @staticmethod
    def _text(value):
        return clean_html(value).strip() if isinstance(value, str) else None

    def _channel_info(self, user, username):
        handle = user.get('username') or username
        name = self._text(user.get('name')) or handle
        user_id = str(user['id']) if user.get('id') is not None else None
        return {
            'uploader': name, 'uploader_id': handle,
            'channel': name, 'channel_id': user_id,
            'uploader_url': f'https://onlyfans.com/{handle}',
            'channel_url': f'https://onlyfans.com/{handle}',
        }

    def _cast(self, post):
        # Release-form users are participant records. linkedUsers can be ads;
        # mentionedUsers can be ordinary mentions, so neither establishes cast.
        names = []
        for form in post.get('releaseForms') or []:
            if not isinstance(form, dict):
                continue
            user = form.get('user')
            if not isinstance(user, dict):
                user = form if form.get('type') == 'user' or form.get('username') else {}
            name = self._text(user.get('name')) or self._text(user.get('username'))
            if name and name not in names:
                names.append(name)
        return names or None

    def _entries_from_posts(self, posts, username, user=None, videos_only=False):
        for post in posts:
            post_id = str(post.get('id') or '')
            if not post_id:
                continue
            author = post.get('author')
            author = author if isinstance(author, dict) else {}
            channel = self._channel_info({**(user or {}), **author}, username)
            post_url = f'https://onlyfans.com/{post_id}/{channel["uploader_id"]}'
            description = self._text(post.get('text')) or self._text(post.get('rawText'))
            title_text = self._text(post.get('rawText')) or description
            title = title_text.splitlines()[0] if title_text else f'{channel["channel"]} post {post_id}'
            timestamp = (int_or_none(post.get('postedAtPrecise'))
                         or unified_timestamp(post.get('postedAt')))
            media_items = traverse_obj(post, ('media', ..., {dict})) or []
            for index, media in enumerate(media_items, 1):
                kind = media.get('type') or media.get('mediaType')
                if videos_only and kind != 'video' and not media.get('convertedToVideo'):
                    continue
                formats = self._extract_formats_from_media(media, post_id)
                if not formats:
                    continue
                media_id = str(media.get('id') or f'{post_id}-{index}')
                yield {
                    **channel,
                    'id': media_id,
                    'display_id': post_id,
                    'title': title,
                    'webpage_url': post_url,
                    'description': description,
                    'cast': self._cast(post),
                    'timestamp': timestamp,
                    'duration': float_or_none(media.get('duration')),
                    'thumbnail': url_or_none(media.get('preview') or media.get('thumb')),
                    'age_limit': 18,
                    'formats': formats,
                }

    def _profile_entries(self, url, username, user, section):
        query = {'limit': 50, 'order': 'publish_date_desc', 'skip_users': 'all', 'format': 'infinite'}
        endpoint = f'/users/{user["id"]}/posts' + ('/videos' if section == 'videos' else '')
        seen = set()
        previous_cursor = None
        page = 1
        while True:
            response = self._download_api_json(endpoint, username, f'Downloading posts page {page}', url, query=query)
            posts = response.get('list') if isinstance(response, dict) else None
            if not posts:
                return
            fresh = [post for post in posts if isinstance(post, dict) and str(post.get('id')) not in seen]
            seen.update(str(post.get('id')) for post in fresh)
            yield from self._entries_from_posts(fresh, username, user, videos_only=section == 'videos')
            if not response.get('hasMore'):
                return
            # Browser fetchUserPosts uses beforePublishTime for descending pages.
            # Keep decimal precision; a float can round away the cursor boundary.
            cursors = []
            # Browser setMarkers treats decimal-string markers as publish times;
            # integer markers can be IDs, so do not reinterpret those as dates.
            markers = [response.get(key) for key in ('headMarker', 'tailMarker')]
            values = [value for value in markers if isinstance(value, str) and '.' in value]
            if not values:
                values = [post.get('postedAtPrecise') for post in posts if isinstance(post, dict)]
            for value in values:
                try:
                    number = Decimal(str(value))
                    if number.is_finite() and number > 0:
                        cursors.append(number)
                except InvalidOperation:
                    pass
            if not cursors:
                raise ExtractorError('OnlyFans returned more posts without a publication cursor.', expected=True)
            cursor = min(cursors)
            if not fresh or (previous_cursor is not None and cursor >= previous_cursor):
                raise ExtractorError('OnlyFans pagination stopped advancing; stopping repeated requests.', expected=True)
            query['beforePublishTime'] = format(cursor, '.6f')
            previous_cursor = cursor
            page += 1

    def _extract_profile(self, url, username, section='posts'):
        user = self._download_api_json(f'/users/{username}', username, 'Downloading profile metadata', url)
        if not int_or_none(user.get('id')):
            raise ExtractorError('OnlyFans did not return a usable profile id', expected=True)
        return self.playlist_result(
            self._profile_entries(url, username, user, section),
            playlist_id=str(user['id']),
            playlist_title=self._text(user.get('name')) or user.get('username') or username,
            playlist_description=self._text(user.get('about')),
            **self._channel_info(user, username))

    def _extract_post(self, url, username, post_id):
        post = self._download_api_json(
            f'/posts/{post_id}', post_id, 'Downloading post metadata', url,
            query={'skip_users': 'all'})
        author = post.get('author')
        user = author if isinstance(author, dict) else {}
        if not user.get('username') or not user.get('name'):
            user = self._download_api_json(
                f'/users/{username}', username, 'Downloading creator metadata', url)
        entries = list(self._entries_from_posts([post], username, user))
        if not entries:
            raise ExtractorError('No downloadable media found in this OnlyFans post', expected=True)
        if len(entries) == 1:
            return entries[0]
        return self.playlist_result(entries, playlist_id=post_id,
                                    playlist_title=entries[0]['title'],
                                    playlist_description=entries[0]['description'],
                                    **self._channel_info(user, username))

    def _real_extract(self, url):
        path_parts = [part for part in urlparse(url).path.split('/') if part]
        username = path_parts[0]
        post_id = None
        if len(path_parts) == 2 and path_parts[0].isdigit():
            post_id, username = path_parts
        elif len(path_parts) == 3 and path_parts[1] in ('post', 'posts', 'video', 'videos') and path_parts[2].isdigit():
            post_id = path_parts[2]
        elif len(path_parts) == 2 and path_parts[1].isdigit():
            post_id = path_parts[1]
        if post_id:
            return self._extract_post(url, username, post_id)
        if len(path_parts) > 2 or (len(path_parts) == 2 and path_parts[1] not in ('posts', 'videos')):
            raise ExtractorError('Unsupported OnlyFans profile section.', expected=True)
        return self._extract_profile(url, username, path_parts[1] if len(path_parts) == 2 else 'posts')
