from kitty.fast_data_types import Screen, wcswidth
from kitty.tab_bar import DrawData, ExtraData, TabBarData, as_rgb
from kitty.utils import color_as_int


MAX_TITLE_CELLS = 10
ALERT_DOT = "󱐌"
INDEX_FG = as_rgb(0x7A86D1)
ALERT_FG = as_rgb(0x09FF00)


SUPERSCRIPT = str.maketrans(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-=()",
    "ᵃᵇᶜᵈᵉᶠᵍʰⁱʲᵏˡᵐⁿᵒᵖqʳˢᵗᵘᵛʷˣʸᶻ"
    "ᴬᴮᶜᴰᴱᶠᴳᴴᴵᴶᴷᴸᴹᴺᴼᴾQᴿˢᵀᵁⱽᵂˣʸᶻ"
    "⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾",
)


def notification_marker(draw_data: DrawData, tab: TabBarData) -> str:
    return ALERT_DOT if tab.needs_attention or tab.has_activity_since_last_focus else " "


def fit_text(text: str, max_cells: int) -> str:
    text = "".join(text.split())
    if wcswidth(text) <= max_cells:
        return text

    ans = ""
    for ch in text:
        if wcswidth(ans + ch + "…") > max_cells:
            break
        ans += ch
    return ans + "…"


def icon_and_title(title: str) -> tuple[str, str]:
    parts = title.split(maxsplit=1)
    if len(parts) == 2:
        return parts[0], parts[1]
    return "", title


def draw_colored(screen: Screen, text: str, fg: int) -> None:
    old_fg = screen.cursor.fg
    screen.cursor.fg = fg
    screen.draw(text)
    screen.cursor.fg = old_fg


def draw_tab(
    draw_data: DrawData,
    screen: Screen,
    tab: TabBarData,
    before: int,
    max_tab_length: int,
    index: int,
    is_last: bool,
    extra_data: ExtraData,
) -> int:
    marker = notification_marker(draw_data, tab)
    icon, raw_title = icon_and_title(tab.title)
    title = fit_text(raw_title, MAX_TITLE_CELLS)
    index_text = str(index).translate(SUPERSCRIPT)

    screen.draw(" ")
    if icon:
        screen.draw(icon)
        screen.draw(" ")
    screen.draw(title)

    # if marker == " ":
    #     screen.draw("")
    # else:
    #     draw_colored(screen, marker, ALERT_FG)

    draw_colored(screen, index_text, INDEX_FG)
    screen.draw(" ")

    end = screen.cursor.x

    screen.cursor.bold = False
    screen.cursor.italic = False
    screen.cursor.fg = 0

    if not is_last:
        screen.cursor.bg = as_rgb(color_as_int(draw_data.inactive_bg))
        screen.draw(draw_data.sep)
        screen.cursor.bg = 0

    return end
