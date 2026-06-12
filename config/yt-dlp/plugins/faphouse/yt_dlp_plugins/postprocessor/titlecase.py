import re

from yt_dlp.postprocessor.common import PostProcessor


def _caps_case(value):
    # capitalize the first letter of each word, leave the rest untouched
    # so existing caps survive (POV stays POV, not Pov)
    return re.sub(r'\S+', lambda m: m.group()[:1].upper() + m.group()[1:], value)


class TitlecasePP(PostProcessor):
    """Adds *_cc (Caps Case) variants of text fields for use in output
    templates, e.g. %(title_cc,title)s. Run with when=pre_process so the
    fields exist before filenames are computed."""

    _FIELDS = ('title', 'channel', 'uploader')

    def run(self, info):
        for field in self._FIELDS:
            value = info.get(field)
            if isinstance(value, str) and value:
                info[f'{field}_cc'] = _caps_case(value)
        return [], info
