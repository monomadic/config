import json
import re
import urllib.parse
import xml.etree.ElementTree as etree

from yt_dlp.extractor.common import InfoExtractor
from yt_dlp.utils import (
    ExtractorError,
    int_or_none,
    str_or_none,
    traverse_obj,
    url_or_none,
)

# Threads only server-renders post data for search-engine crawlers; a normal
# browser UA gets a JS shell with no media in it.
_CRAWLER_UA = 'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)'


class _ThreadsBaseIE(InfoExtractor):
    _CRAWLER_HEADERS = {'User-Agent': _CRAWLER_UA}


class ThreadsIE(_ThreadsBaseIE):
    IE_NAME = 'threads'
    _VALID_URL = r'https?://(?:www\.)?threads\.(?:com|net)/(?:@(?P<user>[^/?#]+)/post|t)/(?P<id>[\w-]+)'
    _TESTS = [{
        'url': 'https://www.threads.com/@vitekvisuals/post/DcRKPf8jGq3',
        'info_dict': {
            'id': 'DcRKPf8jGq3',
            'title': r're:.+',
            'uploader': 'vitekvisuals',
        },
        'playlist_mincount': 3,
    }, {
        'url': 'https://www.threads.net/t/DcRKPf8jGq3',
        'only_matching': True,
    }]

    def _sjs_blocks(self, webpage):
        for block in re.findall(
                r'<script[^>]+\btype="application/json"[^>]*>(.*?)</script>', webpage, re.DOTALL):
            try:
                yield json.loads(block)
            except ValueError:
                continue

    def _iter_posts(self, data):
        """Yield every dict in the blob that looks like a Threads media post."""
        stack = [data]
        while stack:
            node = stack.pop()
            if isinstance(node, dict):
                if node.get('pk') and (
                        node.get('carousel_media') or node.get('video_versions')
                        or node.get('image_versions2')):
                    yield node
                stack.extend(node.values())
            elif isinstance(node, list):
                stack.extend(node)

    def _find_post(self, webpage, video_id):
        # Match on `code` rather than taking the first hit: the page also carries
        # the carousel children, related posts and profile media.
        for data in self._sjs_blocks(webpage):
            for post in self._iter_posts(data):
                if post.get('code') == video_id:
                    return post

    def _extract_formats(self, media):
        formats = []

        dash_manifest = media.get('video_dash_manifest')
        if dash_manifest:
            try:
                formats.extend(self._parse_mpd_formats(
                    etree.fromstring(dash_manifest.encode()), mpd_id='dash'))
            except (etree.ParseError, ValueError) as e:
                self.report_warning(f'Failed to parse DASH manifest: {e}')

        # video_versions repeats one progressive file under several `type` codes,
        # and that file is usually the highest DASH representation served whole.
        # Key on the path (the query carries per-request auth) to keep a single
        # copy, and only as a fallback for when the manifest is missing.
        seen = {urllib.parse.urlparse(fmt['url']).path for fmt in formats}
        for version in traverse_obj(media, ('video_versions', lambda _, v: url_or_none(v['url']))):
            video_url = version['url']
            path = urllib.parse.urlparse(video_url).path
            if path in seen:
                continue
            seen.add(path)
            formats.append({
                'url': video_url,
                'format_id': f'progressive-{version["type"]}' if version.get('type') else 'progressive',
                'ext': 'mp4',
                'quality': -2,
            })

        return formats

    def _extract_thumbnails(self, media):
        return traverse_obj(media, ('image_versions2', 'candidates', lambda _, v: url_or_none(v['url']), {
            'url': 'url',
            'width': ('width', {int_or_none}),
            'height': ('height', {int_or_none}),
        }))

    def _media_entry(self, media, media_id, common):
        formats = self._extract_formats(media)
        if not formats:
            return None
        return {
            **common,
            'id': media_id,
            'formats': formats,
            'thumbnails': self._extract_thumbnails(media),
            'width': int_or_none(media.get('original_width')),
            'height': int_or_none(media.get('original_height')),
        }

    def _real_extract(self, url):
        video_id, user = self._match_valid_url(url).group('id', 'user')
        webpage_url = (
            f'https://www.threads.com/@{user}/post/{video_id}' if user
            else f'https://www.threads.com/t/{video_id}')

        webpage = self._download_webpage(webpage_url, video_id, headers=self._CRAWLER_HEADERS)
        post = self._find_post(webpage, video_id)
        if not post:
            raise ExtractorError(
                'Could not find post data; the post may be private or deleted', expected=True)

        uploader = traverse_obj(post, ('user', 'username', {str_or_none}))
        full_name = traverse_obj(post, ('user', 'full_name', {str_or_none}))
        caption = traverse_obj(post, ('caption', 'text', {str_or_none}))
        hashtags = re.findall(r'#([^\s#.,;:!?]+)', caption) if caption else []
        common = {
            'display_id': video_id,
            'title': caption or (f'Threads post by {uploader}' if uploader else video_id),
            'description': caption,
            'tags': hashtags or None,
            'timestamp': int_or_none(post.get('taken_at')),
            'uploader': uploader,
            'uploader_id': traverse_obj(post, ('user', 'pk', {str_or_none})),
            'uploader_url': f'https://www.threads.com/@{uploader}' if uploader else None,
            'channel': uploader,
            'channel_id': traverse_obj(post, ('user', 'pk', {str_or_none})),
            'creators': [full_name] if full_name else None,
            'like_count': int_or_none(post.get('like_count')),
            'comment_count': traverse_obj(
                post, ('text_post_app_info', 'direct_reply_count', {int_or_none})),
            'repost_count': traverse_obj(
                post, ('text_post_app_info', 'repost_count', {int_or_none})),
            'webpage_url': (
                f'https://www.threads.com/@{uploader}/post/{video_id}' if uploader else webpage_url),
        }

        carousel = post.get('carousel_media')
        if carousel:
            entries = []
            for idx, media in enumerate(carousel, 1):
                entry = self._media_entry(media, f'{video_id}_{idx}', common)
                if entry:
                    entry['title'] = f'{common["title"]} ({idx})'
                    entries.append(entry)
            if not entries:
                raise ExtractorError('This post contains no videos', expected=True)
            if len(entries) == 1:
                entries[0]['id'] = video_id
                entries[0]['title'] = common['title']
                return entries[0]
            return self.playlist_result(entries, video_id, common['title'], caption)

        entry = self._media_entry(post, video_id, common)
        if not entry:
            raise ExtractorError('This post contains no videos', expected=True)
        return entry


class ThreadsShareIE(_ThreadsBaseIE):
    IE_NAME = 'threads:share'
    _VALID_URL = r'https?://(?:www\.)?threads\.(?:com|net)/share/(?P<id>[\w-]+)'
    _TESTS = [{
        'url': 'https://www.threads.com/share/_gmU2kosX/',
        'only_matching': True,
    }]

    def _real_extract(self, url):
        share_id = self._match_id(url)
        urlh = self._request_webpage(
            url, share_id, 'Resolving share link', headers=self._CRAWLER_HEADERS)
        resolved = urlh.url
        if ThreadsIE.suitable(resolved):
            return self.url_result(resolved, ThreadsIE)
        raise ExtractorError(f'Share link resolved to an unsupported URL: {resolved}', expected=True)
