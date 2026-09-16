# ~/.config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/pimpbunny.py

import functools
import re
import urllib.parse

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    int_or_none,
    js_to_json,
    OnDemandPagedList,
    parse_duration,
    parse_resolution,
    remove_end,
    unescapeHTML,
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

    # Quality labels change once you are signed in: the top rendition is served
    # as `1440p` to guests but `2K` to logged-in users, and parse_resolution()
    # knows `4K`/`8K` but not `2K`. The /get_file/ path still carries `_1440p`
    # so height normally survives via the URL, but this keeps the label alone
    # sufficient if that ever stops being true.
    _LABEL_HEIGHTS = {'2k': 1440, 'qhd': 1440}

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

    def _extract_formats(self, config, url, webpage):
        formats, seen = [], set()
        for key in sorted(filter(re.compile(r'^video_(?:url|alt_url\d*)$').match, config)):
            video_url = config.get(key)
            if not isinstance(video_url, str) or '/get_file/' not in video_url:
                continue
            format_id = config.get(f'{key}_text') or key
            resolution = parse_resolution(format_id) or parse_resolution(video_url)
            if not resolution.get('height'):
                height = self._LABEL_HEIGHTS.get(format_id.strip().lower())
                resolution = {'height': height} if height else resolution
            formats.append({
                'url': urljoin(url, self._kvs_real_url(video_url, config.get('license_code'))),
                'format_id': format_id,
                'ext': 'mp4',
                **resolution,
                # /get_file/ 302s to a CDN host and rejects requests whose session
                # cookie does not match the one the page was served under; yt-dlp
                # reuses its cookiejar, so only the Referer needs restating.
                'http_headers': self._headers(url),
            })
            if not formats[-1].get('height'):
                formats[-1]['quality'] = 1
            seen.add(urllib.parse.urlparse(video_url).path.rstrip('/').rsplit('/', 1)[-1])

        formats.extend(self._extract_download_formats(webpage, url, seen))
        return formats

    def _extract_download_formats(self, webpage, url, seen):
        """
        Subscriber accounts get a download popover listing the renditions as
        direct links. It is usually the same set the player already carries, but
        not always — one sampled video exposes a 240p here that the player omits
        entirely — so these are merged in rather than ignored.

        Guests get `data-dialog="upgrade"` instead of this popover, so nothing is
        found and nothing breaks.

        Note this is NOT a route to the `_source.mkv` master that `preview_url*`
        hints at: that file is absent from the popover on every video checked,
        including ones whose source is taller than the best published rendition.
        """
        block = self._search_regex(
            r'data-popover-name="download-actions"(.*?)</ul>',
            webpage, 'download actions', default=None, flags=re.S)
        if not block:
            return []

        formats = []
        for link, label in re.findall(r'<a[^>]+href="([^"]+)"[^>]*>([^<]+)</a>', block):
            link = url_or_none(unescapeHTML(link))
            if not link or '/get_file/' not in link:
                continue
            # Dedupe against the player formats on the rendition filename
            # (581372_1440p.mp4) — the query string carries a per-request token,
            # so the full URL is never comparable.
            path = urllib.parse.urlparse(link).path.rstrip('/').rsplit('/', 1)[-1]
            if path in seen:
                continue
            seen.add(path)

            format_id = re.sub(r'^\s*MP4\s+', '', clean_html(label)).strip() or path
            resolution = parse_resolution(format_id) or parse_resolution(path)
            if not resolution.get('height'):
                height = self._LABEL_HEIGHTS.get(format_id.lower())
                resolution = {'height': height} if height else resolution
            formats.append({
                'url': link,
                'format_id': format_id,
                'format_note': 'subscriber download',
                'ext': 'mp4',
                **resolution,
                'http_headers': self._headers(url),
            })
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
        formats = self._extract_formats(config, url, webpage)
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


class PimpBunnyPlaylistIE(InfoExtractor):
    """
    Index/gallery pages: OnlyFans creators, categories and tags.

    Pagination is path-based (`/tags/asian/2/`), 32 videos per page. Two traps
    make the obvious "scrape every /videos/ link" approach wrong:

      1. Out-of-range pages answer **HTTP 200**, not 404 — the body is a
         "Page Not Found" document that still carries a dozen *recommended*
         video links in a substitute `list_videos_popular_videos_items` grid.
         Following those would append junk entries and never terminate.
      2. Even valid pages surround the gallery with sidebar links.

    So entries are read only from the real gallery grid (`<div id="..._items">`,
    bounded by the pagination block that follows it), and a page whose title
    says "Page Not Found" terminates the walk.
    """

    IE_NAME = 'pimpbunny:playlist'
    IE_DESC = 'PimpBunny creator/category/tag galleries'
    _VALID_URL = (
        r'https?://(?:www\.)?pimpbunny\.com/'
        r'(?P<kind>onlyfans-creators|categories|tags)/'
        r'(?P<id>[^/?#&]+)(?:/\d+)?/?(?:[?#]|$)')
    _PAGE_SIZE = 32
    _HEADERS = PimpBunnyIE._HEADERS

    _TESTS = [{
        'url': 'https://pimpbunny.com/onlyfans-creators/thiccasianbaddie/',
        'info_dict': {
            'id': 'thiccasianbaddie',
            'title': 'ThiccAsianBaddie',
        },
        'playlist_mincount': 5,
    }, {
        # Multi-page gallery; a trailing page number is normalised away so the
        # whole gallery is enumerated rather than just that one page.
        'url': 'https://pimpbunny.com/tags/asian/2/',
        'info_dict': {
            'id': 'asian',
        },
        'playlist_mincount': 100,
    }, {
        'url': 'https://pimpbunny.com/categories/teen/',
        'info_dict': {
            'id': 'teen',
        },
        'playlist_mincount': 100,
    }]

    def _headers(self, url):
        return {**self._HEADERS, 'Referer': url}

    def _is_not_found(self, webpage):
        return 'Page Not Found' in (self._html_extract_title(webpage, default='') or '')

    def _gallery_html(self, webpage):
        """
        Narrow the page down to the gallery grid.

        The grid is `<div id="<block>_items">` and is always followed by
        `<... id="<block>_pagination">`, which is used as the closing delimiter —
        the grid nests too deeply to match its own `</div>` with a regex.
        """
        start = re.search(r'id="[^"]*_items"', webpage)
        if not start:
            return None
        rest = webpage[start.end():]
        end = re.search(r'id="[^"]*_pagination"', rest)
        return rest[:end.start()] if end else rest

    def _parse_entries(self, webpage):
        gallery = self._gallery_html(webpage)
        if not gallery:
            return []

        entries = []
        # Class names carry a per-build hash suffix (ui-card-link__KxRw6l), so
        # every selector here matches on the stable prefix only.
        for card in re.split(r'(?=<div[^>]+class="[^"]*ui-card-root)', gallery)[1:]:
            video_url = url_or_none(self._search_regex(
                r'<a[^>]+class="[^"]*ui-card-link[^"]*"[^>]+href="([^"]+)"',
                card, 'video url', default=None))
            if not video_url or '/videos/' not in video_url:
                continue

            thumbnail = url_or_none(self._search_regex(
                r'data-original="([^"]+)"', card, 'thumbnail', default=None))
            entries.append(self.url_result(
                video_url, ie=PimpBunnyIE.ie_key(),
                # The numeric id is recoverable from the screenshot path, which
                # keeps --flat-playlist output matching the video extractor's ids.
                video_id=self._search_regex(
                    r'/videos_screenshots/\d+/(\d+)/', thumbnail or '', 'id', default=None),
                video_title=clean_html(self._search_regex(
                    r'<div[^>]+class="[^"]*ui-card-title[^"]*"[^>]*>([^<]+)</div>',
                    card, 'title', default=None) or '') or None,
                thumbnail=thumbnail,
                duration=parse_duration(self._search_regex(
                    r'<div[^>]+class="[^"]*ui-card-duration[^"]*"[^>]*>([^<]+)</div>',
                    card, 'duration', default=None)),
                age_limit=18))
        return entries

    def _fetch_page(self, base_url, playlist_id, first_page, page):
        if page == 0:
            webpage = first_page
        else:
            page_url = f'{base_url}{page + 1}/'
            webpage = self._download_webpage(
                page_url, playlist_id, headers=self._headers(page_url),
                note=f'Downloading gallery page {page + 1}', fatal=False)
            # An out-of-range page is a 200 with a "Page Not Found" body, so the
            # walk has to stop on content, not on status.
            if not webpage or self._is_not_found(webpage):
                return []
        return self._parse_entries(webpage)

    def _playlist_title(self, webpage, playlist_id):
        # The first <h1> is the age-gate banner ("This site is for adults only!");
        # the gallery's own heading is the next one.
        for heading in re.findall(r'<h1[^>]*>(.*?)</h1>', webpage, re.S):
            heading = clean_html(heading)
            if heading and 'adults only' not in heading.lower():
                return heading
        title = self._og_search_title(webpage, default=None) or self._html_extract_title(
            webpage, default=None)
        if not title:
            return playlist_id
        # "Name OnlyFans Leaks | 5 Videos | PimpBunny, Page 2" -> "Name OnlyFans Leaks"
        title = re.sub(r',\s*Page\s*\d+\s*$', '', clean_html(title))
        return title.split('|')[0].strip() or playlist_id

    def _real_extract(self, url):
        kind, playlist_id = self._match_valid_url(url).group('kind', 'id')
        # A trailing page number is dropped: pasting page 5 of a gallery should
        # still yield the whole gallery, which --playlist-items can then slice.
        base_url = f'https://pimpbunny.com/{kind}/{playlist_id}/'

        first_page = self._download_webpage(
            base_url, playlist_id, headers=self._headers(base_url),
            note='Downloading gallery page 1')
        if self._is_not_found(first_page):
            raise ExtractorError(f'No such {kind} gallery: {playlist_id}', expected=True)

        return self.playlist_result(
            OnDemandPagedList(
                functools.partial(self._fetch_page, base_url, playlist_id, first_page),
                self._PAGE_SIZE),
            playlist_id=playlist_id,
            playlist_title=self._playlist_title(first_page, playlist_id),
            playlist_count=int_or_none(self._search_regex(
                r'class="[^"]*includes-pagination-count[^"]*"[^>]*>\s*([\d,]+)\s*videos',
                first_page, 'video count', default='').replace(',', '')) or None)
