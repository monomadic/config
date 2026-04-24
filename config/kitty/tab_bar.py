from kitty.fast_data_types import Screen
from kitty.tab_bar import DrawData, ExtraData, TabBarData, as_rgb, draw_title
from kitty.utils import color_as_int


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
    """
    Separator-style tab renderer that behaves like Kitty's stock renderer,
    except truncated tabs end with '… ' instead of just '…'.

    This preserves normal tab sizing. The extra padding is only introduced
    when a tab is actually ellipsized.
    """

    # Match stock separator renderer.
    if draw_data.leading_spaces:
        screen.draw(" " * draw_data.leading_spaces)

    draw_title(draw_data, screen, tab, index, max_tab_length)

    trailing_spaces = min(max_tab_length - 1, draw_data.trailing_spaces)
    available = max_tab_length - trailing_spaces
    extra = screen.cursor.x - before - available

    if extra > 0:
        # Normal Kitty behavior writes just '…' at:
        #   screen.cursor.x -= extra + 1
        #
        # We want '… ' instead, while keeping the same total width.
        # So move back one extra cell and draw two cells instead of one.
        if available >= 2:
            screen.cursor.x -= extra + 2
            screen.draw("… ")
        else:
            screen.cursor.x -= extra + 1
            screen.draw("…")
    elif trailing_spaces:
        screen.draw(" " * trailing_spaces)

    end = screen.cursor.x

    screen.cursor.bold = False
    screen.cursor.italic = False
    screen.cursor.fg = 0

    if not is_last:
        screen.cursor.bg = as_rgb(color_as_int(draw_data.inactive_bg))
        screen.draw(draw_data.sep)
        screen.cursor.bg = 0

    return end
