from yt_dlp.extractor.common import InfoExtractor
from urllib.parse import urlparse


class XVideosRedIE(InfoExtractor):
    IE_NAME = "xvideos:red"
    _VALID_URL = r'https?://(?:www\.)?xvideos\.red/video\.[^/?#]+/.+'

    def _real_extract(self, url):
        p = urlparse(url)

        # keep path exactly, just change domain
        fixed = f"https://www.xvideos.com{p.path}"

        return self.url_result(fixed, ie_key="XVideos")
