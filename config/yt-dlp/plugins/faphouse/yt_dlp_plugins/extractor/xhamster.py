import re
from html import unescape

from yt_dlp.extractor.xhamster import XHamsterIE


class XHamsterEnrichedIE(XHamsterIE):
    # Regular plugin extractor (no plugin_name=): yt-dlp prepends regular plugin
    # classes to the extractor lookup, so this matches xhamster video URLs before
    # the built-in XHamsterIE while still subclassing it to reuse _real_extract.
    # (The plugin_name= override mechanism is a no-op here because this build uses
    # lazy_extractors, whose registry never sees the patched module attribute.)
    IE_NAME = 'xhamster:enriched'

    # The built-in extractor pulls formats + basic metadata from the embedded
    # window.initials JSON, which carries no tags/categories/performers. Those
    # live only as rendered HTML "chip" links in the page. Scrape them here.
    #
    # Video-specific chips all carry a `tag-<hash>` CSS-module class; the global
    # navigation links to the same path types use `itemContent-*` instead, so
    # keying on `tag-<hash>` excludes the nav menu. The URL path segment tells us
    # what the chip is: /tags/ -> tags, /categories/ -> categories,
    # /creators/ and /pornstars/ -> cast.
    _CHIP_RE = re.compile(
        r'<a\b[^>]*\bclass="[^"]*\btag-[0-9a-z]+\b[^"]*"[^>]*'
        r'href="https?://xhamster\.com/(tags|categories|creators|pornstars)/([^"/?#]+)"'
        r'[^>]*>(.*?)</a>', re.S)
    _LABEL_RE = re.compile(r'<span\b[^>]*\blabel-[0-9a-z]+\b[^>]*>(.*?)</span>', re.S)
    _BUCKET = {
        'tags': 'tags',
        'categories': 'categories',
        'creators': 'cast',
        'pornstars': 'cast',
    }

    @staticmethod
    def _clean(html):
        html = re.sub(r'<!--.*?-->', '', html, flags=re.S)
        html = re.sub(r'<[^>]+>', '', html)
        return unescape(re.sub(r'\s+', ' ', html)).strip()

    def _chip_name(self, slug, inner):
        # Prefer the chip's visible label span; the leading char is an avatar
        # initial for creator chips, so reading the whole anchor would prepend it.
        label = self._LABEL_RE.search(inner)
        name = self._clean(label.group(1) if label else inner)
        return name or slug.replace('-', ' ').title()

    def _real_extract(self, url):
        info = super()._real_extract(url)

        webpage = self._download_webpage(
            url, info.get('id') or info.get('display_id'),
            note='Downloading webpage for tags/categories/cast',
            fatal=False) or ''

        buckets = {'tags': [], 'categories': [], 'cast': []}
        for seg, slug, inner in self._CHIP_RE.findall(webpage):
            bucket = buckets[self._BUCKET[seg]]
            name = self._chip_name(slug, inner)
            if name not in bucket:
                bucket.append(name)

        for key, values in buckets.items():
            if values:
                info[key] = values

        # Present the same identity as the built-in extractor so %(extractor)s in
        # filenames and the download-archive key ("xhamster <id>") are unchanged.
        # IE_NAME stays unique for the registry; add_extra_info only setdefaults
        # these, so pinning them here wins.
        info['extractor'] = XHamsterIE.IE_NAME
        info['extractor_key'] = XHamsterIE.ie_key()

        return info
