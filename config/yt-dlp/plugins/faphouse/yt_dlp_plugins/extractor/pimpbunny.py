# ~/.config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/pimpbunny.py

import re
import urllib.parse

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    int_or_none,
    js_to_json,
    parse_resolution,
    remove_end,
    traverse_obj,
    url_or_none,
    urljoin,
)


class PimpBunnyIE(InfoExtractor):
    """
    PimpBunny runs Kernel Video Sharing (KVS) player v4.

    yt-dlp's generic extractor already knows KVS, but it cannot handle this site
    for two reasons:

      1. The player config object is emitted under a *randomised* variable name
         (`var t5e7bb91412 = {...}`) rather than the stock `var flashvars = {...}`,
         so `GenericIE._extract_kvs` bails with "Unable to extract flashvars".
      2. Even if it parsed, the generic title regex would pick up the age-gate
         banner — the page's only <h1> is "This site is for adults only!".

    Both are handled here: the config variable is located via the `kt_player(...)`
    call that consumes it, and the title comes from the player config.
    """

    IE_NAME = 'pimpbunny'
    IE_DESC = 'PimpBunny'
    _VALID_URL = r'https?://(?:www\.)?pimpbunny\.com/(?:videos|embed)/(?P<id>[^/?#&]+)'
    _HEADERS = {
        'User-Agent': (
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) '
            'AppleWebKit/537.36 (KHTML, like Gecko) '
            'Chrome/147.0.0.0 Safari/537.36'
        ),
    }

    _TESTS = [{
        'url': 'https://pimpbunny.com/videos/michellefromchina18yes-gets-pussy-destroyed/',
        'info_dict': {
            'id': '484027',
            'display_id': 'michellefromchina18yes-gets-pussy-destroyed',
            'ext': 'mp4',
            'title': 'MichelleFromChina18Yes Gets Pussy Destroyed',
            'thumbnail': r're:https?://pimpbunny\.com/contents/.+\.jpg',
            'duration': 3287,
            'upload_date': '20251220',
            'uploader': 'slaslaslamm',
            'cast': ['MichelleFromChina18Yes'],
            'age_limit': 18,
        },
        # The media URLs carry a session-bound `v-acctoken`, so a cached page
        # cannot be replayed; skip the actual transfer.
        'params': {'skip_download': True},
    }]

    def _headers(self, url):
        return {**self._HEADERS, 'Referer': url}

    # -- KVS URL de-obfuscation -------------------------------------------------
    #
    # Mirrors GenericIE._kvs_get_real_url / _kvs_get_license_token. Vendored
    # rather than imported so the plugin does not depend on yt-dlp's private
    # generic-extractor internals, which move between releases. The algorithm is
    # fixed by the player engine (v4-v6), so it does not drift.
    #
    # Not currently exercised by pimpbunny — its `video_url` values are plain
    # /get_file/ links — but KVS toggles obfuscation per-site at any time, and
    # without this the extractor would silently start emitting 404 URLs.

    @staticmethod
    def _kvs_license_token(license_code):
        license_code = license_code.replace('$', '')
        license_values = [int(char) for char in license_code]

        modlicense = license_code.replace('0', '1')
        center = len(modlicense) // 2
        fronthalf = int(modlicense[:center + 1])
        backhalf = int(modlicense[center:])
        modlicense = str(4 * abs(fronthalf - backhalf))[:center + 1]

        return [
            (license_values[index + offset] + current) % 10
            for index, current in enumerate(map(int, modlicense))
            for offset in range(4)
        ]

    @classmethod
    def _kvs_real_url(cls, video_url, license_code):
        if not video_url.startswith('function/0/'):
            return video_url  # not obfuscated
        if not license_code:
            raise ExtractorError('Obfuscated stream URL but no license_code in player config')

        parsed = urllib.parse.urlparse(video_url[len('function/0/'):])
        license_token = cls._kvs_license_token(license_code)
        urlparts = parsed.path.split('/')

        HASH_LENGTH = 32
        hash_ = urlparts[3][:HASH_LENGTH]
        indices = list(range(HASH_LENGTH))

        # Swap hash indices according to the destination derived from the token.
        accum = 0
        for src in reversed(range(HASH_LENGTH)):
            accum += license_token[src]
            dest = (src + accum) % HASH_LENGTH
            indices[src], indices[dest] = indices[dest], indices[src]

        urlparts[3] = ''.join(hash_[i] for i in indices) + urlparts[3][HASH_LENGTH:]
        return urllib.parse.urlunparse(parsed._replace(path='/'.join(urlparts)))

    # -- player config ----------------------------------------------------------

    def _player_config(self, webpage, video_id):
        """
        Locate and parse the KVS player config object.

        The page ends the player block with the call that consumes it:

            window['player_obj'] = kt_player(
                'kt_player', 'https://pimpbunny.com/player/kt_player.swf?v=4.12.5',
                '100%', '100%', t5e7bb91412);

        so the last argument names the variable to read. Falling back to a scan
        keeps this working if the call is ever reshaped or minified: any
        `var <name> = {` whose body mentions `video_url` is the config.
        """
        candidates = []

        var_name = self._search_regex(
            r'kt_player\s*\([^)]*?,\s*(\w+)\s*\)', webpage,
            'player config variable', default=None)
        if var_name:
            candidates.append(var_name)
        candidates.append('flashvars')  # stock KVS name

        for match in re.finditer(r'var\s+(\w+)\s*=\s*\{', webpage):
            # Bounded lookahead: the config is a single large object literal, and
            # `video_url` sits near its head.
            if 'video_url' in webpage[match.end():match.end() + 20000]:
                candidates.append(match.group(1))

        for name in dict.fromkeys(candidates):
            config = self._search_json(
                rf'var\s+{re.escape(name)}\s*=\s*', webpage, 'player config', video_id,
                transform_source=js_to_json, default=None, fatal=False)
            if config and config.get('video_url'):
                return config

        raise ExtractorError(
            'Could not locate the KVS player config object. The page layout likely '
            'changed; re-check the kt_player() call in the page source.',
            expected=True, video_id=video_id)

    def _extract_formats(self, config, url):
        formats = []
        for key in sorted(filter(re.compile(r'^video_(?:url|alt_url\d*)$').match, config)):
            video_url = config.get(key)
            if not isinstance(video_url, str) or '/get_file/' not in video_url:
                continue
            format_id = config.get(f'{key}_text') or key
            formats.append({
                'url': urljoin(url, self._kvs_real_url(video_url, config.get('license_code'))),
                'format_id': format_id,
                'ext': 'mp4',
                **(parse_resolution(format_id) or parse_resolution(video_url)),
                # /get_file/ 302s to a CDN host and rejects requests whose session
                # cookie does not match the one the page was served under; yt-dlp
                # reuses its cookiejar, so only the Referer needs restating.
                'http_headers': self._headers(url),
            })
            if not formats[-1].get('height'):
                formats[-1]['quality'] = 1
        return formats

    def _comma_list(self, value):
        if not isinstance(value, str):
            return None
        items = []
        for item in value.split(','):
            item = clean_html(item).strip()
            if item and item not in items:
                items.append(item)
        return items or None

    def _real_extract(self, url):
        display_id = self._match_id(url)
        webpage = self._download_webpage(url, display_id, headers=self._headers(url))

        config = self._player_config(webpage, display_id)
        formats = self._extract_formats(config, url)
        if not formats:
            raise ExtractorError(
                'No /get_file/ stream URLs in the player config', expected=True,
                video_id=display_id)

        json_ld = self._search_json_ld(webpage, display_id, default={})

        title = clean_html(
            config.get('video_title')
            or self._og_search_title(webpage, default=None)
            # Deliberately NOT <h1>: the page's only <h1> is the age-gate banner.
            or remove_end(self._html_extract_title(webpage, default='') or '', ' | PimpBunny')
        ) or display_id

        # og:description is the *category* blurb, not the video's, and JSON-LD
        # usually just repeats the title — keep neither in those cases.
        description = clean_html(json_ld.get('description') or '') or None
        if description == title:
            description = None

        uploader = clean_html(self._search_regex(
            r'<a[^>]+href="[^"]*/members/\d+/?"[^>]*class="[^"]*pages-view-video-uploaded-name[^"]*"[^>]*>([^<]+)</a>',
            webpage, 'uploader', default=None) or '') or None

        thumbnail = url_or_none(urljoin(url, config.get('preview_url'))) or url_or_none(
            json_ld.get('thumbnail') or self._og_search_thumbnail(webpage, default=None))

        return {
            'id': str(config.get('video_id') or display_id),
            'display_id': display_id,
            'title': title,
            'description': description,
            'thumbnail': thumbnail,
            'uploader': uploader,
            'duration': int_or_none(json_ld.get('duration')),
            'upload_date': json_ld.get('upload_date'),
            'timestamp': json_ld.get('timestamp'),
            'view_count': int_or_none(json_ld.get('view_count')),
            'like_count': int_or_none(json_ld.get('like_count')),
            'cast': self._comma_list(config.get('video_models')),
            'categories': self._comma_list(config.get('video_categories')),
            'tags': self._comma_list(config.get('video_tags')),
            'age_limit': 18,
            'formats': formats,
        }
