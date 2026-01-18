# ~/.config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/faphouse.py

import re

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    traverse_obj,
    url_or_none,
)


class FaphouseIE(InfoExtractor):
    IE_NAME = "faphouse"
    _VALID_URL = r'''(?x)
        https?://(?:www\.)?faphouse\.com/
        (?:
            videos|video|watch
        )/
        (?P<id>[^/?#&]+)
    '''

    _M3U8_RE = r'(https?://[^\s"\'<>]+\.m3u8[^\s"\'<>]*)'

    def _download_webpage_fallback(self, url, video_id):
        try:
            return self._download_webpage(url, video_id)
        except ExtractorError as e:
            cause = getattr(e, "cause", None)
            # best-effort 404 check
            if not (cause and "404" in str(cause)):
                raise
            
        url_www = re.sub(r'^https?://(?:www\.)?', 'https://www.', url)
        return self._download_webpage(
            url_www, video_id,
            headers={
                "Referer": url_www,
                "User-Agent": (
                    "Mozilla/5.0 (Macintosh; ARM Mac OS X) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    "Chrome/120 Safari/537.36"
                ),
            },
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
        data = (
            self._search_json(
                r'__NEXT_DATA__\s*=\s*',
                webpage, "next data", video_id,
                contains_pattern=r'\{.+\}', end_pattern=r'</script>', fatal=False
            )
            or self._search_json(
                r'window\.__NUXT__\s*=\s*',
                webpage, "nuxt data", video_id,
                contains_pattern=r'\{.+\}', end_pattern=r'</script>', fatal=False
            )
            or {}
        )

        # Best-effort traversal for an HLS URL in those blobs
        m3u8 = (
            traverse_obj(data, (..., "sources", ..., "src"), get_all=False)
            or traverse_obj(data, (..., "hls"), get_all=False)
            or traverse_obj(data, (..., "m3u8"), get_all=False)
        )
        m3u8 = url_or_none(m3u8)

        # Fallback: brute regex for any m3u8 in HTML
        if not m3u8:
            m3u8 = url_or_none(self._search_regex(self._M3U8_RE, webpage, "m3u8 url", default=None))

        if not m3u8:
            raise ExtractorError(
                "Could not find HLS playlist URL (m3u8). "
                "Use browser DevTools Network to locate the JSON/XHR that returns the HLS URL(s), "
                "then implement _download_json() here.",
                expected=True,
            )

        formats = self._extract_m3u8_formats(
            m3u8,
            video_id,
            ext="mp4",
            m3u8_id="hls",
            headers={"Referer": url},
            fatal=False,
        )

        return {
            "id": video_id,
            "title": title,
            "channel": channel,
            "description": description,
            "cast": cast if cast else None,
            "tags": tags if tags else None,
            "formats": formats,
        }


class FaphouseModelIE(InfoExtractor):
    IE_NAME = "faphouse:model"
    _VALID_URL = r'https?://(?:www\.)?faphouse\.com/(?:models|creators?)/(?P<id>[^/?#&]+)'

    # Accept:
    #  - /videos/ABC123
    #  - /videos/some-slug-ABC123
    _VIDEO_PATH_RE = re.compile(r'^/videos/(?:[A-Za-z0-9]{6}|.+-[A-Za-z0-9]{6})$')

    def _download_webpage_fallback(self, url, page_id):
        try:
            return self._download_webpage(url, page_id)
        except ExtractorError as e:
            cause = getattr(e, "cause", None)
            if not (cause and "404" in str(cause)):
                raise
        url_www = re.sub(r'^https?://(?:www\.)?', 'https://www.', url)
        return self._download_webpage(
            url_www, page_id,
            headers={
                "Referer": url_www,
                "User-Agent": (
                    "Mozilla/5.0 (Macintosh; ARM Mac OS X) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    "Chrome/120 Safari/537.36"
                ),
            },
        )

    def _real_extract(self, url):
        model_id = self._match_id(url)
        webpage = self._download_webpage_fallback(url, model_id)

        # HTML scrape (works only if links are in server-rendered HTML)
        video_paths = set(re.findall(r'href="(/videos/[^"?#]+)"', webpage))
        video_paths = {p for p in video_paths if self._VIDEO_PATH_RE.match(p)}

        if not video_paths:
            raise ExtractorError(
                "No video links found in model/creator page HTML. "
                "This page is probably JS-rendered; implement the model API pagination endpoint instead.",
                expected=True,
            )

        entries = [
            self.url_result(f"https://faphouse.com{p}", ie=FaphouseIE.ie_key())
            for p in sorted(video_paths)
        ]

        title = self._og_search_title(webpage, default=None)
        if title:
            title = clean_html(title)

        return self.playlist_result(entries, playlist_id=model_id, playlist_title=title or model_id)
