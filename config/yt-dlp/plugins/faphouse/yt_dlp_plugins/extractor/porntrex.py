# ~/.config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/porntrex.py

import re
import urllib.parse

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    datetime_from_str,
    int_or_none,
    js_to_json,
    parse_duration,
    parse_filesize,
    parse_resolution,
    url_or_none,
    urljoin,
)


class PornTrexIE(InfoExtractor):
    """
    PornTrex runs a stock Kernel Video Sharing (KVS) player, so yt-dlp's generic
    extractor already finds the renditions. What it does not get is everything
    around them: the title it reads is the <title> tag, which the site truncates
    with "..." at ~50 characters, and there is no uploader, cast, date, duration
    or description. All of that sits in the page's "video-info" block, which is
    what this extractor reads.

    Quirks worth knowing:

      - The site publishes only *relative* dates ("23 hours ago", "10 months
        ago") and nothing absolute, not even to logged-in users. The timestamp
        here is derived from that string and so is only as precise as the unit
        shown; a video "10 months ago" gets the first of that month.
      - `/embed/<id>` pages carry the player config but none of the info block,
        and `/video/<id>` without its slug is an empty 200, so an embed (or
        slug-less) URL is resolved through the canonical link the embed page
        carries, and the full video page is what gets read.
      - The download popover is the only place the renditions' sizes appear.
        Guests see the labels with their sizes but the links point at /login/,
        so sizes are matched onto the player formats by height rather than
        taken as formats of their own.
      - The /get_file/ links 302 to an expiring CDN URL. They are not obfuscated
        today, but KVS can switch that on per site, so the standard
        `function/0/` de-obfuscation is kept here (vendored from the generic
        extractor's private helpers so a yt-dlp release can't move them).
    """

    IE_NAME = 'porntrex'
    IE_DESC = 'PornTrex'
    _VALID_URL = r'https?://(?:www\.)?porntrex\.com/(?:video|embed)/(?P<id>\d+)(?:/(?P<display_id>[^/?#&]+))?'
    _HEADERS = {
        'User-Agent': (
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) '
            'AppleWebKit/537.36 (KHTML, like Gecko) '
            'Chrome/147.0.0.0 Safari/537.36'
        ),
    }

    _TESTS = [{
        'url': 'https://www.porntrex.com/video/3362642/yasmina-khan-manuel-ferrara-finally-got-his-hands-on-me-4k',
        'info_dict': {
            'id': '3362642',
            'display_id': 'yasmina-khan-manuel-ferrara-finally-got-his-hands-on-me-4k',
            'ext': 'mp4',
            'title': 'Yasmina Khan - MANUEL FERRARA FINALLY GOT HIS HANDS ON ME! 4k',
            'description': r're:for latest onlyfans content',
            'thumbnail': r're:https?://ptx\.cdntrex\.com/contents/videos_screenshots/.+\.jpg',
            'duration': 1747,
            'uploader': 'king6364',
            'uploader_id': '3999917',
            'uploader_url': 'https://www.porntrex.com/members/3999917/',
            'cast': ['Yasmina Khan'],
            'categories': ['Interracial', 'Indian', 'BWC (Big White Cock)'],
            'age_limit': 18,
        },
        # Stream URLs expire and carry a per-session hash; skip the transfer.
        'params': {'skip_download': True},
    }, {
        'url': 'https://www.porntrex.com/embed/3362642',
        'only_matching': True,
    }]

    def _headers(self, url):
        return {**self._HEADERS, 'Referer': url}

    # -- KVS URL de-obfuscation (mirrors GenericIE._kvs_get_real_url) ----------

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

        accum = 0
        for src in reversed(range(HASH_LENGTH)):
            accum += license_token[src]
            dest = (src + accum) % HASH_LENGTH
            indices[src], indices[dest] = indices[dest], indices[src]

        urlparts[3] = ''.join(hash_[i] for i in indices) + urlparts[3][HASH_LENGTH:]
        return urllib.parse.urlunparse(parsed._replace(path='/'.join(urlparts)))

    # -- page pieces ------------------------------------------------------------

    def _player_config(self, webpage, video_id):
        config = self._search_json(
            r'var\s+flashvars\s*=\s*', webpage, 'player config', video_id,
            transform_source=js_to_json, default=None)
        if not config or not config.get('video_url'):
            raise ExtractorError(
                'Could not locate the KVS player config (flashvars). The page layout '
                'likely changed.', expected=True, video_id=video_id)
        return config

    def _download_sizes(self, webpage):
        """
        Height -> bytes, from the download popover:

            <li><a ... href="https://www.porntrex.com/login/">MP4 2160p 4k, 3.14 Gb</a></li>
        """
        block = self._search_regex(
            r'class="btn-favourites download"(.*?)</ul>', webpage,
            'download popover', default='', flags=re.S)
        sizes = {}
        for label in re.findall(r'<a[^>]*>([^<]+)</a>', block):
            height = parse_resolution(label).get('height')
            # The site's "Mb"/"Gb" are binary (a "305.81 Mb" file transfers as
            # 305.81 MiB), which parse_filesize would read as decimal.
            size = parse_filesize(re.sub(
                r'(?i)\b([KMGT])b\b', r'\1iB', label.rpartition(',')[2].strip()))
            if height and size:
                sizes[height] = size
        return sizes

    def _extract_formats(self, config, url, webpage):
        sizes = self._download_sizes(webpage)
        formats = []
        for key in sorted(filter(re.compile(r'^video_(?:url|alt_url\d*)$').match, config)):
            video_url = config.get(key)
            if not isinstance(video_url, str) or '/get_file/' not in video_url:
                continue
            label = config.get(f'{key}_text') or key
            resolution = parse_resolution(label) or parse_resolution(video_url)
            formats.append({
                'url': urljoin(url, self._kvs_real_url(video_url, config.get('license_code'))),
                'format_id': label,
                'ext': 'mp4',
                **resolution,
                'filesize_approx': sizes.get(resolution.get('height')),
                # /get_file/ 302s to the CDN; the hop wants the page's Referer
                # and session cookie (yt-dlp replays the jar on its own).
                'http_headers': self._headers(url),
            })
        return formats

    def _abs_url(self, url, value):
        # Thumbnails are protocol-relative (`//ptx.cdntrex.com/...`); without
        # pinning the scheme yt-dlp's sanitiser would turn them into http://.
        return url_or_none(self._proto_relative_url(urljoin(url, value), 'https:'))

    def _thumbnails(self, config, url):
        thumbnails = []
        for key, value in config.items():
            m = re.fullmatch(r'preview_url(\d*)', key)
            thumb = self._abs_url(url, value) if m else None
            if not thumb:
                continue
            thumbnails.append({
                'url': thumb,
                'height': int_or_none(config.get(f'preview_height{m.group(1)}')),
                # The bare preview.jpg is the poster the site itself uses.
                'preference': 1 if not m.group(1) else None,
            })
        return thumbnails

    def _info_block(self, webpage):
        return self._search_regex(
            r'<div class="video-info">(.*?)<div id="tab_report_rrror"', webpage,
            'video info block', default='', flags=re.S)

    def _badge(self, block, icon):
        return clean_html(self._search_regex(
            rf'<i class="fa fa-{icon}"></i>\s*<em class="badge">([^<]*)</em>',
            block, icon, default='') or '').strip() or None

    def _links(self, block, section, path):
        """Names from `<span class="title-item">Models:</span> ... <a href=".../models/x/">`."""
        chunk = self._search_regex(
            rf'<span class="title-item">{section}:</span>(.*?)</div>\s*</div>',
            block, section, default='', flags=re.S)
        names = []
        for name in re.findall(rf'<a[^>]+href="[^"]*/{path}/[^"]+"[^>]*>(.*?)</a>', chunk, re.S):
            name = clean_html(name).strip()
            if name and name not in names:
                names.append(name)
        return names or None

    def _timestamp(self, relative):
        """
        '23 hours ago' / '10 months ago' -> epoch. Only as precise as the unit:
        datetime_from_str truncates to it, which is the honest reading.
        """
        m = re.fullmatch(r'(\d+)\s*(second|minute|hour|day|week|month|year)s?\s+ago', relative or '')
        if not m:
            return None
        try:
            return int(datetime_from_str(f'now-{m.group(1)}{m.group(2)}s', precision='auto').timestamp())
        except ValueError:
            return None

    def _real_extract(self, url):
        video_id, display_id = self._match_valid_url(url).group('id', 'display_id')
        if not display_id:
            # `/video/<id>` without its slug is an empty 200 and `/embed/<id>`
            # carries the player but no info block, so the slug is recovered
            # from the embed page, which links the canonical video URL.
            embed_url = f'https://www.porntrex.com/embed/{video_id}'
            embed_page = self._download_webpage(
                embed_url, video_id, headers=self._headers(embed_url), note='Resolving video URL')
            display_id = self._search_regex(
                rf'https?://(?:www\.)?porntrex\.com/video/{video_id}/([^/"\'?#\s]+)',
                embed_page, 'display id')
        page_url = f'https://www.porntrex.com/video/{video_id}/{display_id}'
        webpage = self._download_webpage(page_url, video_id, headers=self._headers(page_url))

        config = self._player_config(webpage, video_id)
        formats = self._extract_formats(config, page_url, webpage)
        if not formats:
            raise ExtractorError(
                'No /get_file/ stream URLs in the player config', expected=True, video_id=video_id)

        block = self._info_block(webpage)

        title = clean_html(
            config.get('video_title')
            or self._og_search_title(webpage, default=None)
            or self._search_regex(r'<p class="title-video">([^<]+)</p>', block, 'title', default=None)
        ) or display_id or video_id

        description = clean_html(self._search_regex(
            r'<div class="videodesc item">.*?<div class="items-holder">(.*?)</div>',
            block, 'description', default='', flags=re.S)).strip() or None

        uploader_url, uploader = self._search_regex(
            r'<div class="username">\s*<a href="([^"]*/members/\d+/?)"[^>]*>([^<]+)</a>',
            block, 'uploader', default=(None, None), group=(1, 2))
        uploader = clean_html(uploader or '').strip() or None

        def comma_list(value):
            return [v.strip() for v in value.split(',') if v.strip()] if isinstance(value, str) else None

        return {
            'id': str(config.get('video_id') or video_id),
            'display_id': display_id,
            'title': title,
            'description': description,
            'thumbnail': self._abs_url(page_url, config.get('preview_url')),
            'thumbnails': self._thumbnails(config, page_url),
            'uploader': uploader,
            'uploader_id': self._search_regex(r'/members/(\d+)', uploader_url or '', 'uploader id', default=None),
            'uploader_url': url_or_none(uploader_url),
            'duration': parse_duration(self._badge(block, 'clock-o')),
            'view_count': int_or_none(re.sub(r'\D', '', self._badge(block, 'eye') or '') or None),
            'timestamp': self._timestamp(self._badge(block, 'calendar')),
            'cast': self._links(block, 'Models', 'models'),
            'categories': self._links(block, 'Categories', 'categories') or comma_list(config.get('video_categories')),
            'tags': self._links(block, 'Tags', 'tags') or comma_list(config.get('video_tags')),
            'age_limit': 18,
            'formats': formats,
        }
