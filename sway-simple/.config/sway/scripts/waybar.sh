#!/bin/sh
# wrapper script for waybar with args, see https://github.com/swaywm/sway/issues/5724

USER_CONFIG_PATH=$HOME/.config/sway/apps/waybar/config.jsonc
USER_STYLE_PATH=$HOME/.config/sway/apps/waybar/style.css

waybar -c ${USER_CONFIG_PATH} -s ${USER_STYLE_PATH} &
