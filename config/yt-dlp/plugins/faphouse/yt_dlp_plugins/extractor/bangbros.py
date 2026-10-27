# ~/.config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/bangbros.py

import re

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.networking import Request
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    int_or_none,
    parse_iso8601,
    parse_resolution,
    url_or_none,
)
from yt_dlp.utils.traversal import traverse_obj


class BangBrosIE(InfoExtractor):
    """
    BangBros runs the Gamma/Project1 "Juan" SPA, shared by every site on the
    project1service platform.

    Metadata comes from the platform's JSON API rather than the page, because
    the page has none to give: `window.__JUAN.initialState` ships with its
    `entities` and `queries` caches **empty** ({}), and the scene record is
    fetched client-side after hydration. What *is* rendered into the HTML is
    unusable as a primary source for two reasons:

      1. Every class name is a per-build styled-components hash
         (`class="sc-1b6bgon-3 jAsNxx"`), so any selector keyed on one breaks
         at the next deploy.
      2. The cast markup is not unique to the scene. The same
         `<a href="/model/...">` shape appears in each of the ~13 related-video
         cards further down the page, so a page-wide scan silently mixes other
         scenes' performers into the cast.

    So the real work is one call to:

        GET https://site-api.project1service.com/v2/releases/<id>
        Instance: <jwt>

    which returns title, description, release date, duration, cast, tags,
    collections and stats as structured JSON.

    Only metadata is in scope here. The full scene is behind a subscription:
    a guest sees just the trailer and the `mediabook` preview loop, which are
    what this extractor offers as formats so that extraction yields a usable
    entry (`--print filename`, `--load-info-json` and metadata post-processors
    all need at least one format). They are labelled as previews rather than
    passed off as the scene.
    """

    IE_NAME = 'bangbros'
    IE_DESC = 'BangBros'
    _VALID_URL = r'''(?x)
        https?://(?:www\.)?bangbros\.com/
        (?:
            # canonical: /video/8904821/breaking-and-entering-in-that-pussy
            (?:video|movie)/(?P<id>\d+)(?:/(?P<display_id>[^/?#&]+))?
            |
            # legacy, still redirected by the site's own route table:
            #   /video/<slug>_<id>  and  /video/<slug>_<id>.htm*
            video/(?P<legacy_display_id>[^/?#&]+?)_(?P<legacy_id>\d+)(?:\.html?\w*)?
            |
            # legacy: /video<id>/<slug>
            video(?P<old_id>\d+)/(?P<old_display_id>[^/?#&]+)
        )
    '''

    _API_URL = 'https://site-api.project1service.com/v2/releases/%s'
    _HEADERS = {
        'User-Agent': (
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) '
            'AppleWebKit/537.36 (KHTML, like Gecko) '
            'Chrome/147.0.0.0 Safari/537.36'
        ),
    }

    _TESTS = [{
        'url': 'https://bangbros.com/video/8904821/breaking-and-entering-in-that-pussy',
        'info_dict': {
            'id': '8904821',
            'display_id': 'breaking-and-entering-in-that-pussy',
            'ext': 'mp4',
            'title': 'Breaking And Entering In that Pussy',
            'description': r're:^Kailani Kai is just at home taking a shower',
            'thumbnail': r're:https?://media-public-ht\.project1content\.com/.+\.jpg',
            'duration': 2372,
            'upload_date': '20200116',
            'timestamp': 1579132800,
            'uploader': 'BangBros',
            'channel': 'BangBros',
            'series': 'Big Tits, Round Asses',
            'cast': ['Kailani Kai', 'Jaxxx Slayher'],
            'age_limit': 18,
        },
        # Only the trailer/preview is free, so the media that would be fetched
        # is not the scene the metadata describes.
        'params': {'skip_download': True},
    }, {
        # A `type: trailer` release: its own `videos.full` *is* the trailer,
        # and it carries no cast or tags of its own.
        'url': 'https://bangbros.com/video/8906511/mini-the-big-dick-slayer',
        'info_dict': {
            'id': '8906511',
            'display_id': 'mini-the-big-dick-slayer',
            'ext': 'mp4',
            'title': 'Mini The Big Dick Slayer',
            'duration': 105,
            'age_limit': 18,
        },
        'params': {'skip_download': True},
    }, {
        # Legacy slug_id form from the site's route table.
        'url': 'https://bangbros.com/video/breaking-and-entering-in-that-pussy_8904821',
        'only_matching': True,
    }, {
        'url': 'https://bangbros.com/video8904821/breaking-and-entering-in-that-pussy',
        'only_matching': True,
    }]

    def _headers(self, url):
        return {**self._HEADERS, 'Referer': url}

    def _match_ids(self, url):
        """Coalesce the canonical and legacy URL shapes into (id, display_id)."""
        groups = self._match_valid_url(url).groupdict()
        return (
            groups['id'] or groups['legacy_id'] or groups['old_id'],
            groups['display_id'] or groups['legacy_display_id']
            or groups['old_display_id'])

    def _instance_token(self, url, video_id, webpage):
        """
        The API rejects an unauthenticated request with HTTP 400, so the
        `Instance` JWT is mandatory. The site hands it out two ways: as an
        `instance_token` cookie on *any* response, and as `jwt` inside
        `window.__JUAN.config`.

        The cookie is what is used, via a HEAD request, because the page is a
        ~1.8 MB SPA shell that carries the token at 99% of its length — so
        reading it from the body means paying for the whole document, and a
        truncated transfer (which this host does produce: `IncompleteRead`
        after 140 kB) can never contain it. HEAD answers in about a second
        with no body at all.

        The token is short-lived (its `exp` sits ~2 days out) and brand-scoped
        (`"brand":"bangbros"`), so it is fetched per extraction, not cached.
        """
        self._request_webpage(
            Request(url, method='HEAD'), video_id,
            note='Fetching instance token',
            errnote='Failed to fetch instance token', fatal=False,
            headers=self._headers(url))
        cookie = self._get_cookies(url).get('instance_token')
        if cookie and cookie.value:
            return cookie.value

        # No cookie: fall back to the copy in the app config, which costs the
        # full page but keeps the extractor working if the cookie is dropped.
        config = self._search_json(
            r'window\.__JUAN\.config\s*=\s*', webpage(), 'app config', video_id,
            default=None, fatal=False)
        return traverse_obj(config, ('jwt', {str})) or self._search_regex(
            r'"jwt"\s*:\s*"([\w-]+\.[\w-]+\.[\w-]+)"', webpage(),
            'instance token', default=None)

    def _call_api(self, video_id, url, token):
        return traverse_obj(self._download_json(
            self._API_URL % video_id, video_id,
            note='Downloading release metadata',
            errnote='Failed to download release metadata',
            headers={
                **self._headers(url),
                'Instance': token,
                'Origin': 'https://bangbros.com',
            }, fatal=False), ('result', {dict}))

    # -- formats ---------------------------------------------------------------

    def _video_formats(self, video, url, note):
        """
        Turn one API `videos.<kind>` object into formats.

        `files` arrives as a dict keyed by quality label on some releases and
        as a plain list on others (a `type: trailer` release uses the list
        form), so both are normalised here.
        """
        files = traverse_obj(video, 'files') or []
        formats = []
        for f in (files.values() if isinstance(files, dict) else files):
            if not isinstance(f, dict):
                continue
            media_url = url_or_none(traverse_obj(f, ('urls', 'view', {str})))
            if not media_url:
                continue
            label = f.get('format') or f.get('label') or ''
            formats.append({
                'url': media_url,
                'format_id': f'{note}-{label}' if label else note,
                'format_note': note,
                'ext': 'mp4',
                'filesize': int_or_none(f.get('sizeBytes')),
                'fps': int_or_none(f.get('fps')),
                'vcodec': f.get('codec'),
                **parse_resolution(label),
                'http_headers': self._headers(url),
            })
        return formats

    def _extract_formats(self, result, url):
        """
        Collect whatever media the account is entitled to.

        A guest gets only previews: `videos.mediabook` (a short looping teaser)
        on the scene itself, plus a `type: trailer` child release carrying the
        real trailer. `videos.full` on a scene is the paywalled scene and only
        appears for a subscriber, so it is preferred when present.
        """
        formats = []
        videos = traverse_obj(result, ('videos', {dict})) or {}
        if videos.get('full'):
            formats.extend(self._video_formats(videos['full'], url, 'full'))
        if videos.get('mediabook'):
            formats.extend(self._video_formats(videos['mediabook'], url, 'preview'))

        for child in traverse_obj(result, ('children', ..., {dict})):
            if child.get('type') == 'trailer':
                trailer = traverse_obj(child, ('videos', 'full', {dict}))
                if trailer:
                    formats.extend(self._video_formats(trailer, url, 'trailer'))

        # Previews must never outrank the scene when a subscriber has both.
        for fmt in formats:
            if fmt['format_note'] != 'full':
                fmt['preference'] = -2
        return formats

    def _extract_thumbnails(self, result):
        """
        `images.poster` is keyed by variant index ('0'...'5') and then by size
        ('xs'...'xx'), but every variant holds the same six URLs, so they are
        deduped rather than emitted six times over.
        """
        thumbnails, seen = [], set()
        for group in ('poster', 'card_main_rect'):
            variants = traverse_obj(result, ('images', group, {dict})) or {}
            for variant in variants:
                for size, image in (traverse_obj(
                        variants, (variant, {dict})) or {}).items():
                    thumb_url = url_or_none(traverse_obj(image, ('url', {str})))
                    if not thumb_url or thumb_url in seen:
                        continue
                    seen.add(thumb_url)
                    thumbnails.append({
                        'id': f'{group}-{size}',
                        'url': thumb_url,
                        'width': int_or_none(image.get('width')),
                        'height': int_or_none(image.get('height')),
                    })
        return thumbnails

    # -- fallback --------------------------------------------------------------

    def _fallback_info(self, webpage, video_id):
        """
        If the API call fails, the server-rendered JSON-LD VideoObject still
        carries title, description, poster, release date and the view/like
        counts — enough to tag a file, which is what this extractor is for.
        """
        json_ld = self._search_json_ld(webpage, video_id, default={})
        title = clean_html(
            json_ld.get('title') or self._html_search_meta(
                ('og:title', 'title'), webpage, default=None))
        if not title:
            return None
        self.report_warning(
            "Release API unavailable; falling back to the page's JSON-LD, "
            'so cast, tags and duration will be missing')
        return {
            'id': video_id,
            'title': title,
            'description': clean_html(json_ld.get('description')) or None,
            'thumbnail': url_or_none(json_ld.get('thumbnail')),
            'timestamp': json_ld.get('timestamp'),
            'upload_date': json_ld.get('upload_date'),
            'view_count': int_or_none(json_ld.get('view_count')),
            'like_count': int_or_none(json_ld.get('like_count')),
            'age_limit': 18,
        }

    # -- main ------------------------------------------------------------------

    def _real_extract(self, url):
        video_id, display_id = self._match_ids(url)

        # The page is only ever a fallback (for the token, then for metadata),
        # so it is fetched lazily and at most once. Held in a local rather than
        # on self: one extractor instance serves every URL in a playlist.
        cache = {}

        def webpage():
            if 'html' not in cache:
                cache['html'] = self._download_webpage(
                    url, video_id, headers=self._headers(url),
                    note='Downloading webpage', fatal=False) or ''
            return cache['html']

        token = self._instance_token(url, video_id, webpage)
        result = self._call_api(video_id, url, token) if token else None

        if not result:
            fallback = self._fallback_info(webpage(), video_id)
            if not fallback:
                raise ExtractorError(
                    'The release API returned no result and the page carried no '
                    'usable metadata', expected=True, video_id=video_id)
            return {
                **fallback,
                'display_id': display_id,
                'formats': self._extract_formats({}, url),
            }

        # The scene's runtime lives on `videos.mediabook.length`. The mediabook
        # file itself is only a few MB of looping teaser, but its `length`
        # reports the parent scene (verified across several releases: 28-52 min,
        # against 43-80 s for a trailer child), so it must not be read off the
        # trailer instead.
        duration = int_or_none(traverse_obj(
            result, ('videos', 'mediabook', 'length'))) or int_or_none(
            traverse_obj(result, ('videos', 'full', 'length')))

        # The API pads descriptions with a long run of trailing whitespace.
        description = clean_html(result.get('description'))
        description = re.sub(r'\s+', ' ', description).strip() if description else None

        brand = traverse_obj(result, ('brandMeta', 'displayName', {str})) or 'BangBros'
        collections = traverse_obj(result, ('collections', ..., 'name', {str}))

        return {
            'id': str(result.get('id') or video_id),
            'display_id': display_id,
            'title': clean_html(result.get('title')) or display_id or video_id,
            'description': description,
            'duration': duration,
            'timestamp': parse_iso8601(result.get('dateReleased')),
            'uploader': brand,
            'channel': brand,
            'creators': [brand],
            # A "collection" is the sub-site a scene was published under
            # ("Big Tits, Round Asses"), which is the closest thing the
            # platform has to a series.
            'series': collections[0] if collections else None,
            'categories': collections or None,
            'cast': traverse_obj(result, ('actors', ..., 'name', {str})) or None,
            'tags': traverse_obj(
                result, ('tags', lambda _, v: v.get('isVisible'), 'name', {str})) or None,
            'view_count': int_or_none(traverse_obj(result, ('stats', 'views'))),
            'like_count': int_or_none(traverse_obj(result, ('stats', 'likes'))),
            'dislike_count': int_or_none(traverse_obj(result, ('stats', 'dislikes'))),
            'average_rating': int_or_none(traverse_obj(result, ('stats', 'rating'))),
            'age_limit': 18,
            'thumbnails': self._extract_thumbnails(result),
            'formats': self._extract_formats(result, url),
        }
