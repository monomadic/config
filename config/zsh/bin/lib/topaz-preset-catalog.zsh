#!/usr/bin/env zsh

# Shared Topaz preset catalog for the wrappers in this directory.
#
# Presets live as one TOML file per preset under topaz-presets/<type>/ (enhancement,
# interpolation, output, transform, encode, simple). A small Python emitter,
# topaz-presets-emit.py, renders those TOML files back into the legacy
# tab-separated rows every consumer already expects — so to add or change a preset
# you edit a TOML file (named fields, comments, no shell quoting) and nothing else.
# The functions below are thin wrappers over the emitter and keep their historical
# names and output formats:
#
#   topaz_preset_catalog_rows   picker  display  preset_name  preset_flag  filter  ext  video_args  metadata
#   topaz_preset_picker_rows    display  preset_name  preset_flag  filter  ext  video_args  metadata   (filtered by picker)
#   topaz_enhancement_preset_rows  category  display  slug  scales  filter_body  blurb  metadata
#   topaz_preset_insights       slug  kind  ...            (from each enhancement TOML's [insight] table)
#   topaz_interpolation_preset_rows  display  slug  fi_filter  metadata
#   topaz_output_profile_rows   display  slug  output_ext  video_args
#   topaz_transform_preset_rows  display  categories  slug  filter  metadata

# Directory of this catalog, captured at source time, so the TOML tree and emitter
# resolve regardless of the caller's cwd (the file is usually sourced, not run).
typeset -g TOPAZ_PRESETS_LIB_DIR="${${(%):-%x}:A:h}"

# Find a python interpreter with the stdlib `tomllib` (3.11+). The default
# `python3` on macOS is often the system 3.9 (no tomllib), so probe named
# versions too and cache the winner for the rest of the shell session. Override
# with TOPAZ_PRESETS_PYTHON to force a specific interpreter.
topaz_presets_python() {
  emulate -L zsh
  if [[ -n "${TOPAZ_PRESETS_PYTHON:-}" ]]; then
    print -r -- "$TOPAZ_PRESETS_PYTHON"
    return 0
  fi
  local candidate
  for candidate in python3 python3.14 python3.13 python3.12 python3.11 python3.10; do
    command -v "$candidate" >/dev/null 2>&1 || continue
    "$candidate" -c 'import tomllib' >/dev/null 2>&1 || continue
    typeset -g TOPAZ_PRESETS_PYTHON="$candidate"
    print -r -- "$candidate"
    return 0
  done
  return 1
}

# Render one preset type's TOML files into legacy tab-separated rows.
topaz_presets_emit() {
  emulate -L zsh
  local kind="$1"
  local dir="$TOPAZ_PRESETS_LIB_DIR"
  local py
  py="$(topaz_presets_python)" || {
    print -u2 -- "topaz presets: need python3 >= 3.11 (stdlib tomllib) on PATH to read the TOML preset tree"
    return 1
  }
  "$py" "$dir/topaz-presets-emit.py" "$kind" "$dir/topaz-presets"
}

topaz_preset_catalog_rows() {
  topaz_presets_emit catalog
}

topaz_preset_picker_rows() {
  emulate -L zsh
  local picker="$1"
  local row_picker display preset_name preset_flag filter_complex output_ext video_args metadata

  topaz_preset_catalog_rows | while IFS=$'\t' read -r row_picker display preset_name preset_flag filter_complex output_ext video_args metadata; do
    [[ "$row_picker" == "$picker" ]] || continue
    print -r -- "${display}"$'\t'"${preset_name}"$'\t'"${preset_flag}"$'\t'"${filter_complex}"$'\t'"${output_ext}"$'\t'"${video_args}"$'\t'"${metadata}"
  done
}

topaz_parse_preset_row() {
  emulate -L zsh
  local row="$1"

  IFS=$'\t' read -r \
    TOPAZ_PRESET_DISPLAY \
    TOPAZ_PRESET_NAME \
    TOPAZ_PRESET_FLAG \
    TOPAZ_PRESET_FILTER \
    TOPAZ_PRESET_OUTPUT_EXT \
    TOPAZ_PRESET_VIDEO_ARGS \
    TOPAZ_PRESET_METADATA <<< "$row"
}

# Transform rows used by the two-step workflow (topaz-workflow).
topaz_transform_preset_rows() {
  topaz_presets_emit transform
}

# Enhancement preset rows for the mpv interactive render menu.
topaz_enhancement_preset_rows() {
  topaz_presets_emit enhancement
}

# Advanced insight rows for the mpv preset-details companion sheet, sourced from
# each enhancement TOML's [insight] table.
topaz_preset_insights() {
  topaz_presets_emit insights
}

# Interpolation (frame-rate) rows for the mpv workflow.
topaz_interpolation_preset_rows() {
  topaz_presets_emit interpolation
}

# Output profile rows.
topaz_output_profile_rows() {
  topaz_presets_emit output
}
