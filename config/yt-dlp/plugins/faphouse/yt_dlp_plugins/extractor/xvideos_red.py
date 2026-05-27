import re
from urllib.parse import urljoin, urlparse

from yt_dlp.extractor.xvideos import XVideosIE
from yt_dlp.utils import (
    clean_html,
    determine_ext,
    int_or_none,
    traverse_obj,
    url_or_none,
)


class XVideosRedIE(XVideosIE):
    IE_NAME = "xvideos:red"
    _VALID_URL = r'https?://(?:www\.)?xvideos\.red/video\.[^/?#]+/.+'

    def _canonical_url(self, url):
        p = urlparse(url)
        return f"https://www.xvideos.com{p.path}"

    def _clean_name(self, html):
        return clean_html(html or '').strip() or None

    def _extract_people(self, webpage):
        uploader = uploader_id = None

        uploader_block = self._search_regex(
            r'(?s)<li[^>]+class=(["\'])(?=[^"\']*\bmain-uploader\b)[^"\']*\1[^>]*>(?P<block>.*?)</li>',
            webpage, 'uploader block', default=None, group='block')
        if uploader_block:
            uploader_id = self._search_regex(
                r'<a[^>]+href=(["\'])(?P<href>[^"\']+)\1',
                uploader_block, 'uploader id', default=None, group='href')
            if uploader_id:
                uploader_id = uploader_id.strip('/').split('/', 1)[0] or None

            uploader = self._clean_name(self._search_regex(
                r'(?s)<span[^>]+class=(["\'])(?=[^"\']*\bname\b)[^"\']*\1[^>]*>(?P<name>.*?)</span>\s*<span',
                uploader_block, 'uploader', default=None, group='name'))

        uploader_id = uploader_id or self._search_regex(
            r'''(?x) \.setUploaderName\(\s* (?P<q>["']) (?P<uploader> (?:(?! (?P=q) ).)+ ) (?P=q) \s*\)''',
            webpage, 'uploader id', default=None, group='uploader')

        cast = []
        for model_block in re.findall(
                r'(?s)<li[^>]+class=(["\'])(?=[^"\']*\bmodel\b)[^"\']*\1[^>]*>(.*?)</li>',
                webpage):
            model = self._clean_name(self._search_regex(
                r'(?s)<span[^>]+class=(["\'])(?=[^"\']*\bname\b)[^"\']*\1[^>]*>(?P<name>.*?)</span>',
                model_block[1], 'model', default=None, group='name'))
            if model and model not in cast:
                cast.append(model)

        return uploader, uploader_id, cast

    def _extract_download_formats(self, url, video_id):
        download_json = self._download_json(
            urljoin(url, f'/video-download/{video_id}/'), video_id,
            note='Downloading direct download information',
            errnote=False, fatal=False, data=b'', headers={
                'Referer': url,
                'X-Requested-With': 'XMLHttpRequest',
            }) or {}

        if not download_json.get('result'):
            return []

        metadata = traverse_obj(download_json, ('metadata', {dict})) or {}
        data = traverse_obj(download_json, ('data', {dict})) or {}

        format_specs = (
            ('download-4k', 'downloadUrl4k', 'mp4q4kResolution', '4K', 2160),
            ('download-hd', 'downloadUrlHd', 'mp4hdResolution', 'HD', None),
            ('download-sd', 'downloadUrlSd', 'mp4sdResolution', 'SD', None),
            ('download-medium', 'downloadUrl', None, '360p', 360),
            ('download-low', 'downloadUrlLow', None, '240p', 240),
        )

        formats = []
        seen_urls = set()
        for quality, (format_id, data_key, resolution_key, note, fallback_height) in enumerate(reversed(format_specs)):
            format_url = url_or_none(data.get(data_key))
            if not format_url or format_url in seen_urls:
                continue
            seen_urls.add(format_url)

            resolution = metadata.get(resolution_key) if resolution_key else note
            height = int_or_none(self._search_regex(
                r'(\d+)\s*p', resolution or '', 'height', default=None)) or fallback_height

            formats.append({
                'url': format_url,
                'format_id': format_id,
                'ext': determine_ext(format_url, 'mp4'),
                'height': height,
                'format_note': resolution or note,
                'quality': quality,
            })

        return formats

    def _real_extract(self, url):
        fixed_url = self._canonical_url(url)
        info = XVideosIE(self._downloader)._real_extract(fixed_url)
        video_id = info['id']

        webpage = self._download_webpage(
            url, video_id, note='Downloading red webpage for metadata',
            fatal=False) or ''

        direct_formats = self._extract_download_formats(url, video_id)
        if direct_formats:
            self._check_formats(direct_formats, video_id)

        if direct_formats:
            formats = info.get('formats') or []
            hls_formats = [
                fmt for fmt in formats
                if fmt.get('protocol', '').startswith('m3u8') or fmt.get('format_id', '').startswith('hls-')
            ]
            best_direct_height = max((fmt.get('height') or 0 for fmt in direct_formats), default=0)
            best_hls_height = max((fmt.get('height') or 0 for fmt in hls_formats), default=0)
            prefer_direct = best_direct_height and best_direct_height >= best_hls_height

            if prefer_direct:
                for fmt in direct_formats:
                    fmt['preference'] = 100
                for fmt in hls_formats:
                    fmt['preference'] = min(fmt.get('preference') or 0, -100)
            else:
                for fmt in direct_formats:
                    fmt['preference'] = min(fmt.get('preference') or 0, -100)

            info['formats'] = direct_formats + (info.get('formats') or [])

        uploader, uploader_id, cast = self._extract_people(webpage)
        if uploader:
            info.update({
                'uploader': uploader,
                'channel': uploader,
                'creator': uploader,
            })
        if uploader_id:
            info.update({
                'uploader_id': uploader_id,
                'channel_id': uploader_id,
            })
        if cast:
            info['cast'] = cast

        info['webpage_url'] = fixed_url
        return info
