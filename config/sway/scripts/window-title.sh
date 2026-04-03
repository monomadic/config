#!/bin/sh
swaymsg -t subscribe -m '["window"]' | jq -r '.container.name'

