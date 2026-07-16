# ~/.config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/faphouse.py

import re

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    int_or_none,
    traverse_obj,
    unified_strdate,
    url_or_none,
)


class FaphouseIE(InfoExtractor):
    IE_NAME = "faphouse"
    _HOST_RE = r'(?:faphouse\.com|fhaccess\.com)'
    _LOGIN_URL = "https://faphouse.com/#signin"
    _VALID_URL = r'''(?x)
        https?://(?:www\.)?''' + _HOST_RE + r'''/
        (?:[a-z]{2}(?:-[a-z]{2})?/)?   # optional locale prefix, e.g. /vi/ or /pt-br/
        (?:(?:videos?|watch))/
        (?P<id>[^/?#&]+)
    '''

    _M3U8_RE = r'(https?://[^\s"\'<>]+\.m3u8[^\s"\'<>]*)'
    _HEADERS = {
        "User-Agent": (
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/147.0.0.0 Safari/537.36"
        ),
    }

    def _headers(self, url):
        return {
            **self._HEADERS,
            "Referer": url,
        }

    def _download_webpage_fallback(self, url, video_id):
        try:
            return self._download_webpage(
                url, video_id,
                headers=self._headers(url),
            )
        except ExtractorError as e:
            cause = getattr(e, "cause", None)
            # best-effort 404 check
            if not (cause and "404" in str(cause)):
                raise

        url_www = re.sub(r'^https?://(?:www\.)?', 'https://www.', url)
        return self._download_webpage(
            url_www, video_id,
            headers=self._headers(url_www),
        )

    def _real_extract(self, url):
        video_id = self._match_id(url)
        webpage = self._download_webpage_fallback(url, video_id)

        title = (
            self._og_search_title(webpage, default=None)
            or self._html_search_meta(["twitter:title", "title"], webpage, default=None)
            or video_id
        )
        title = clean_html(title) or video_id

        # Extract channel/studio - use (?s) flag for DOTALL
        channel = self._search_regex(
            r'(?s)<a[^>]+class="[^"]*video-info-details__studio-link[^"]*"[^>]*>([^<]+)</a>',
            webpage, 'channel', default=None
        )
        if channel:
            channel = clean_html(channel).strip()

        # Extract description
        description = self._search_regex(
            r'(?s)<div[^>]+class="[^"]*video-info-details__description[^"]*"[^>]*>.*?<p>([^<]+)</p>',
            webpage, 'description', default=None
        )
        if description:
            description = clean_html(description).strip()

        thumbnail = (
            self._og_search_thumbnail(webpage, default=None)
            or self._html_search_meta(
                ["twitter:image", "twitter:image:src", "thumbnail", "thumbnailUrl"],
                webpage, default=None,
            )
        )
        thumbnail = url_or_none(thumbnail)

        # Extract cast/actors (vid-c vid-c_image links)
        cast = re.findall(
            r'(?s)<a[^>]+class="vid-c vid-c_image"[^>]*>.*?<span[^>]+class="[^"]*studio-seo-improvements__category-btn-title[^"]*"[^>]*>\s*([^<]+?)\s*</span>',
            webpage
        )
        if cast:
            cast = [clean_html(actor).strip() for actor in cast if actor.strip()]

        # Extract tags/categories (vid-c links, but NOT vid-c_image)
        tags = re.findall(
            r'<a\s+class="vid-c"\s+href="/c/[^"]+">([^<]+)</a>',
            webpage
        )
        if tags:
            tags = [clean_html(tag).strip() for tag in tags if tag.strip()]

        # Try common embedded JSON blobs first (Next/Nuxt/etc.)
        # Prefer yt-dlp helpers if present (version-dependent), else fallback to script-tag search.
        nextjs = None
        nuxt = None

        _search_next = getattr(self, "_search_nextjs_data", None)
        if callable(_search_next):
            # `fatal=False` still warns on recent yt-dlp; use an explicit default to stay quiet.
            nextjs = _search_next(webpage, video_id, default={})

        _search_nuxt = getattr(self, "_search_nuxt_data", None)
        if callable(_search_nuxt):
            nuxt = _search_nuxt(webpage, video_id, fatal=False)

        # Fallback to generic JSON search, but:
        #  - match actual Next.js embedding (<script id="__NEXT_DATA__" ...>{...}</script>)
        #  - stay quiet when absent (default={})
        if not nextjs:
            nextjs = self._search_json(
                r'<script[^>]+id="__NEXT_DATA__"[^>]*>\s*',
                webpage, "next data", video_id,
                end_pattern=r'\s*</script>',
                default={},
                fatal=False,
            )

        if not nuxt:
            nuxt = self._search_json(
                r'window\.__NUXT__\s*=\s*',
                webpage, "nuxt data", video_id,
                end_pattern=r'</script>',
                default={},
                fatal=False,
            )

        view_state = self._search_json(
            r'<script[^>]+id="view-state-data"[^>]*>\s*',
            webpage, "view state data", video_id,
            end_pattern=r'\s*</script>',
            default={},
            fatal=False,
        )

        paywall = self._search_json(
            r'<script[^>]+id="video-paywall"[^>]*>\s*',
            webpage, "video paywall data", video_id,
            end_pattern=r'\s*</script>',
            default={},
            fatal=False,
        )

        # Structured metadata from the embedded view-state blob. This is present even
        # for paywalled videos the current account cannot watch, and is more reliable
        # than the HTML regexes above, so use it to fill any gaps.
        vstate = traverse_obj(view_state, "video") or {}

        channel = channel or traverse_obj(vstate, "studioName")
        if not cast:
            cast = [
                clean_html(name).strip()
                for name in traverse_obj(vstate, ("pornstarNames", ...)) or []
                if name and name.strip()
            ]
        duration = int_or_none(traverse_obj(vstate, "duration"))
        upload_date = unified_strdate(traverse_obj(vstate, "publishedAt"))

        data = nextjs or nuxt or view_state or paywall or {}

        # Best-effort traversal for an HLS URL in those blobs (try common shapes first)
        m3u8 = (
            traverse_obj(data, ("props", "pageProps", ..., "sources", ..., "src"), get_all=False)
            or traverse_obj(data, ("props", "pageProps", ..., "hls"), get_all=False)
            or traverse_obj(data, ("props", "pageProps", ..., "m3u8"), get_all=False)
            or traverse_obj(data, ("state", ..., "sources", ..., "src"), get_all=False)
            or traverse_obj(data, (..., "sources", ..., "src"), get_all=False)
            or traverse_obj(data, (..., "hls"), get_all=False)
            or traverse_obj(data, (..., "m3u8"), get_all=False)
            or traverse_obj(view_state, (..., "sources", ..., "src"), get_all=False)
            or traverse_obj(view_state, (..., "hls"), get_all=False)
            or traverse_obj(view_state, (..., "m3u8"), get_all=False)
            or traverse_obj(paywall, (..., "sources", ..., "src"), get_all=False)
            or traverse_obj(paywall, (..., "hls"), get_all=False)
            or traverse_obj(paywall, (..., "m3u8"), get_all=False)
        )
        m3u8 = url_or_none(m3u8)

        # Fallback: brute regex for any m3u8 in HTML
        if not m3u8:
            m3u8 = url_or_none(self._search_regex(self._M3U8_RE, webpage, "m3u8 url", default=None))

        formats = []
        if m3u8:
            formats = self._extract_m3u8_formats(
                m3u8,
                video_id,
                ext="mp4",
                m3u8_id="hls",
                headers=self._headers(url),
                fatal=False,
            )
        else:
            # No playable stream. This is expected for paywalled videos the current
            # account cannot watch. The page still carries full metadata, so emit it
            # rather than aborting: raise_no_formats() honours --ignore-no-formats-error,
            # letting `--skip-download --ignore-no-formats-error` (or --write-info-json)
            # capture the metadata, while still failing by default when a download was
            # actually intended.
            is_guest = (
                traverse_obj(paywall, ("user", "isGuest"))
                or traverse_obj(view_state, ("user", "currentUserId")) is None
            )
            has_access = traverse_obj(view_state, ("video", "videoViewAllowed"))
            access_type = traverse_obj(view_state, ("video", "videoAccessTypeLabel"))
            if is_guest:
                msg = (
                    "Faphouse returned a guest page for this request. "
                    f"Sign in at {self._LOGIN_URL}, refresh the page in Brave, "
                    "then retry with fresh browser cookies "
                    "(for example, --cookies-from-browser brave)."
                )
            elif has_access is False:
                msg = (
                    "This Faphouse account does not appear to have access to the full video"
                    f"{f' ({access_type})' if access_type else ''}."
                )
            else:
                msg = (
                    "Could not find HLS playlist URL (m3u8). "
                    "Use browser DevTools Network to locate the JSON/XHR that returns the "
                    "HLS URL(s), then implement _download_json() here."
                )
            self.raise_no_formats(msg, expected=True, video_id=video_id)

        return {
            "id": video_id,
            "title": title,
            "channel": channel,
            "uploader": channel,
            "description": description,
            "thumbnail": thumbnail,
            "duration": duration,
            "upload_date": upload_date,
            "cast": cast if cast else None,
            "tags": tags if tags else None,
            "formats": formats,
        }


class FaphouseModelIE(InfoExtractor):
    IE_NAME = "faphouse-model"
    _VALID_URL = r'''(?x)
         https?://(?:www\.)?''' + FaphouseIE._HOST_RE + r'''/
         (?:[a-z]{2}(?:-[a-z]{2})?/)?   # optional locale prefix
         (?:models|creators?|pornstars)/
         (?P<id>[^/?#&]+)
    '''

    # Accept:
    #  - /videos/ABC123
    #  - /videos/some-slug-ABC123
    _VIDEO_PATH_RE = re.compile(r'^/videos/(?:[A-Za-z0-9]{6}|.+-[A-Za-z0-9]{6})$')

    _HEADERS = FaphouseIE._HEADERS

    def _headers(self, url):
        return {
            **self._HEADERS,
            "Referer": url,
        }

    def _download_webpage_fallback(self, url, page_id):
        try:
            return self._download_webpage(
                url, page_id,
                headers=self._headers(url),
            )
        except ExtractorError as e:
            cause = getattr(e, "cause", None)
            if not (cause and "404" in str(cause)):
                raise

        url_www = re.sub(r'^https?://(?:www\.)?', 'https://www.', url)
        return self._download_webpage(
            url_www, page_id,
            headers=self._headers(url_www),
        )

    def _real_extract(self, url):
        model_id = self._match_id(url)
        webpage = self._download_webpage_fallback(url, model_id)
        origin = self._search_regex(r'^(https?://(?:www\.)?%s)' % FaphouseIE._HOST_RE, url, 'origin')

        # Model pages embed video paths in multiple places, not only in visible anchors.
        raw_video_paths = re.findall(r'/videos/[^"\'?#&\s<>]+', webpage)
        video_paths = list(dict.fromkeys(
            p for p in raw_video_paths if self._VIDEO_PATH_RE.match(p)
        ))

        if not video_paths:
            raise ExtractorError(
                "No video links found in model/creator page HTML.",
                expected=True,
            )

        entries = [
            self.url_result(f"{origin}{p}", ie=FaphouseIE.ie_key())
            for p in video_paths
        ]

        title = self._og_search_title(webpage, default=None)
        if title:
            title = clean_html(title)

        return self.playlist_result(entries, playlist_id=model_id, playlist_title=title or model_id)
