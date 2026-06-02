import re

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    clean_html,
    extract_attributes,
    int_or_none,
    parse_duration,
    parse_iso8601,
    remove_end,
    unescapeHTML,
    url_or_none,
)


class HDSexIE(InfoExtractor):
    IE_NAME = 'hdsex'
    _VALID_URL = r'https?://(?:(?:www|[a-z]{2})\.)?hdsex\.org/(?:video|embed)/(?P<id>\d+)'

    _HEADERS = {
        'User-Agent': (
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) '
            'AppleWebKit/537.36 (KHTML, like Gecko) '
            'Chrome/147.0.0.0 Safari/537.36'
        ),
    }
    _M3U8_RE = r'(https?://[^\s"\'<>]+/stream/[^\s"\'<>]+\.m3u8[^\s"\'<>]*)'

    def _headers(self, url):
        return {
            **self._HEADERS,
            'Referer': url,
        }

    def _clean_text(self, value):
        value = clean_html(value or '')
        return value.strip() or None

    def _clean_list(self, values):
        cleaned = []
        for value in values or []:
            if isinstance(value, dict):
                value = value.get('title') or value.get('name')
            value = self._clean_text(value)
            if value and value not in cleaned:
                cleaned.append(value)
        return cleaned or None

    def _itemprop_value(self, webpage, itemprop):
        for tag in re.findall(r'<(?:meta|link)\b[^>]+>', webpage):
            attrs = extract_attributes(tag)
            if attrs.get('itemprop') == itemprop:
                return attrs.get('content') or attrs.get('href')

    def _player_attrs(self, webpage):
        player_tag = self._search_regex(
            r'(?s)(<app-player\b[^>]*>)', webpage,
            'player data', default=None)
        return extract_attributes(player_tag) if player_tag else {}

    def _video_info(self, player_attrs, video_id):
        raw_info = player_attrs.get(':video-info') or player_attrs.get('video-info')
        if not raw_info:
            return {}
        return self._parse_json(raw_info, video_id, fatal=False) or {}

    def _player_source_url(self, webpage):
        video_source = self._search_regex(
            r'''(?sx)
                <video\b(?=[^>]+\bid=(["'])player\1)[^>]*>
                .*?(<source\b[^>]+>)
            ''',
            webpage, 'player source', default=None, group=2)
        if video_source:
            return extract_attributes(video_source).get('src')

    def _fallback_m3u8_url(self, webpage):
        m3u8 = self._search_regex(self._M3U8_RE, webpage, 'm3u8 url', default=None)
        return unescapeHTML(m3u8) if m3u8 else None

    def _strip_site_suffix(self, title):
        if not title:
            return None
        title = remove_end(title, ' - HDSex.org')
        return re.sub(r'\s*-\s*HDSex\.org$', '', title, flags=re.IGNORECASE).strip()

    def _real_extract(self, url):
        video_id = self._match_id(url)
        webpage_url = url if '/video/' in url else f'https://hdsex.org/video/{video_id}'
        webpage = self._download_webpage(
            webpage_url, video_id, headers=self._headers(webpage_url))

        player_attrs = self._player_attrs(webpage)
        video_info = self._video_info(player_attrs, video_id)

        title = self._strip_site_suffix(self._clean_text(
            video_info.get('title')
            or self._itemprop_value(webpage, 'name')
            or self._og_search_title(webpage, default=None)
            or self._html_search_meta('title', webpage, default=None)))
        title = title or video_id

        description = self._clean_text(
            self._og_search_description(webpage, default=None)
            or self._html_search_meta('description', webpage, default=None))

        thumbnail = url_or_none(
            player_attrs.get('poster')
            or self._itemprop_value(webpage, 'thumbnailUrl')
            or self._itemprop_value(webpage, 'thumbnail')
            or self._og_search_thumbnail(webpage, default=None))

        m3u8_url = url_or_none(
            self._itemprop_value(webpage, 'contentUrl')
            or self._player_source_url(webpage)
            or self._fallback_m3u8_url(webpage))
        if not m3u8_url:
            raise ExtractorError('Could not find HDSex HLS playlist URL', expected=True)

        duration = (
            int_or_none(video_info.get('duration'))
            or int_or_none(player_attrs.get('duration'))
            or int_or_none(parse_duration(self._itemprop_value(webpage, 'duration'))))
        timestamp = (
            parse_iso8601(self._itemprop_value(webpage, 'uploadDate'))
            or int_or_none((video_info.get('created') or {}).get('seconds')))

        uploader = self._clean_text(
            (video_info.get('uploader') or {}).get('name')
            or (video_info.get('uploader') or {}).get('title')
            or video_info.get('fake_user_name')
            or self._search_regex(
                r'(?s)<svg[^>]+>\s*<use[^>]+#download[^>]+>.*?</svg>\s*by:\s*([^<]+)',
                webpage, 'uploader', default=None))

        formats = self._extract_m3u8_formats(
            m3u8_url, video_id, ext='mp4', m3u8_id='hls',
            headers=self._headers(webpage_url), fatal=False)

        return {
            'id': video_info.get('id') or video_id,
            'display_id': video_id,
            'title': title,
            'description': description,
            'duration': duration,
            'timestamp': timestamp,
            'thumbnail': thumbnail,
            'uploader': uploader,
            'channel': uploader,
            'view_count': int_or_none(video_info.get('views')),
            'like_count': int_or_none(video_info.get('likes')),
            'dislike_count': int_or_none(video_info.get('dislikes')),
            'tags': self._clean_list(video_info.get('tags')),
            'categories': self._clean_list(video_info.get('categories')),
            'age_limit': 18,
            'formats': formats,
            'http_headers': self._headers(webpage_url),
        }
