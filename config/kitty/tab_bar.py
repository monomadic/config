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
    Custom separator-style tab renderer that preserves at least one trailing
    padding cell even when the tab gets ellipsized.

    Based on Kitty's draw_tab_with_separator(), but with one intentional change:
    always reserve >= 1 trailing space when possible.
    """

    # Preserve the stock leading padding behavior.
    if draw_data.leading_spaces:
        screen.draw(" " * draw_data.leading_spaces)

    # Draw the title using kitty's own template engine so your
    # tab_title_template / active_tab_title_template still apply.
    draw_title(draw_data, screen, tab, index, max_tab_length)

    # Force at least one trailing space if the tab is wide enough to allow it.
    # Kitty's stock code uses:
    #   trailing_spaces = min(max_tab_length - 1, draw_data.trailing_spaces)
    # We instead keep one cell of right padding alive whenever max_tab_length > 1.
    if max_tab_length > 1:
        trailing_spaces = max(1, min(max_tab_length - 1, draw_data.trailing_spaces))
    else:
        trailing_spaces = 0

    content_budget = max_tab_length - trailing_spaces
    extra = screen.cursor.x - before - content_budget

    # If the rendered title overflowed the available content width,
    # back up and place an ellipsis, but leave the reserved trailing space intact.
    if extra > 0:
        screen.cursor.x -= extra + 1
        screen.draw("…")

    if trailing_spaces:
        screen.draw(" " * trailing_spaces)

    end = screen.cursor.x

    # Draw the inter-tab separator the same way Kitty's separator renderer does.
    screen.cursor.bold = False
    screen.cursor.italic = False
    screen.cursor.fg = 0

    if not is_last:
        screen.cursor.bg = as_rgb(color_as_int(draw_data.inactive_bg))
        screen.draw(draw_data.sep)
        screen.cursor.bg = 0

    return end
