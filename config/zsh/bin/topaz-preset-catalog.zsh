#!/usr/bin/env zsh

# Shared Topaz preset catalog for wrappers in this directory.
# Row format:
#   picker<TAB>display_label<TAB>preset_name<TAB>preset_flag<TAB>filter_complex

topaz_preset_catalog_rows() {
  emulate -L zsh

  print -r -- $'run\t4K Enhance - clean digital look without cartoonish output\t4K Enhance - clean digital look without cartoonish output\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.10:details=0.30:halo=0.02:blur=0.06:compression=0.10:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tProb-4 2x - medium enhancement\tProb-4 2x - medium enhancement\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.35:details=0.60:halo=0:blur=0.20:compression=0.30:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tProb-4 2x - strong detail recovery\tProb-4 2x - strong detail recovery\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.55:details=0.90:halo=0:blur=0.50:compression=0.45:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\t1080p to 4K - balanced upscale\t1080p to 4K - balanced upscale\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.08:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tFocus Fix - mild recovery\tFocus Fix - mild recovery\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.15:details=0.55:halo=0.04:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tFocus Fix - stronger recovery\tFocus Fix - stronger recovery\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.22:details=0.80:halo=0.08:blur=0.28:compression=0.18:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\t30fps to 60fps - Apollo / interpolation\t30fps to 60fps - Apollo / interpolation\t--filter_complex\ttvai_fi=model=apollo:slowmo=0:rdt=0.01:device=0:vram=0.95:instances=1,fps=60'
  print -r -- $'run\t24fps to 60fps - Apollo / interpolation\t24fps to 60fps - Apollo / interpolation\t--filter_complex\ttvai_fi=model=apollo:slowmo=0:rdt=0.01:device=0:vram=0.95:instances=1,fps=60'
  print -r -- $'run\tIris MQ - moderate cleanup and sharpening\tIris MQ - moderate cleanup and sharpening\t--filter_complex\ttvai_up=model=iris-mq:scale=1:preblur=0:noise=0.25:details=0.50:halo=0:blur=0.15:compression=0.20:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tArtemis HQ - gentle enhancement for decent sources\tArtemis HQ - gentle enhancement for decent sources\t--filter_complex\ttvai_up=model=ahq-13:scale=1:preblur=0:noise=0.15:details=0.35:halo=0:blur=0.10:compression=0.15:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tNyx - strong denoise for dark/noisy footage\tNyx - strong denoise for dark/noisy footage\t--filter_complex\ttvai_up=model=nyx-3:scale=1:preblur=0:noise=0.80:details=0.25:halo=0:blur=0.20:compression=0.25:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tProteus manual - balanced recover + denoise\tProteus manual - balanced recover + denoise\t--filter_complex\ttvai_up=model=prob-3:scale=1:preblur=0:noise=0.40:details=0.55:halo=0.05:blur=0.15:compression=0.25:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'

  print -r -- $'simple\t4K Conservative Cleanup\t4k-conservative-cleanup\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.20:details=0.22:halo=0.03:blur=0.08:compression=0.18:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Balanced Enhancement\t4k-balanced-enhancement\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.32:details=0.36:halo=0.05:blur=0.14:compression=0.30:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Aggressive Artifact Cleanup\t4k-aggressive-artifact-cleanup\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.50:details=0.28:halo=0.10:blur=0.18:compression=0.42:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Focus Fix Light\t4k-focus-fix-light\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.12:details=0.26:halo=0.02:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Focus Fix Strong\t4k-focus-fix-strong\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.34:halo=0.03:blur=0.34:compression=0.16:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Clean Upscale\t1080p-to-4k-clean\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.16:details=0.26:halo=0.02:blur=0.08:compression=0.12:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Medium Enhancement\t1080p-to-4k-medium\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.28:details=0.34:halo=0.04:blur=0.12:compression=0.22:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Strong Recovery\t1080p-to-4k-strong\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.42:details=0.44:halo=0.06:blur=0.20:compression=0.30:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t30fps -> 60fps Interpolation\t30fps-to-60fps\t--filter_complex\ttvai_fi=model=chf-3:slowmo=0:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p30 -> 4K60\t1080p30-to-4k60\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.26:details=0.32:halo=0.03:blur=0.10:compression=0.20:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=chf-3:slowmo=0:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
}

topaz_preset_picker_rows() {
  emulate -L zsh
  local picker="$1"
  local row_picker display preset_name preset_flag filter_complex

  topaz_preset_catalog_rows | while IFS=$'\t' read -r row_picker display preset_name preset_flag filter_complex; do
    [[ "$row_picker" == "$picker" ]] || continue
    print -r -- "${display}"$'\t'"${preset_name}"$'\t'"${preset_flag}"$'\t'"${filter_complex}"
  done
}

topaz_parse_preset_row() {
  emulate -L zsh
  local row="$1"

  TOPAZ_PRESET_DISPLAY="${row%%$'\t'*}"
  row="${row#*$'\t'}"
  TOPAZ_PRESET_NAME="${row%%$'\t'*}"
  row="${row#*$'\t'}"
  TOPAZ_PRESET_FLAG="${row%%$'\t'*}"
  TOPAZ_PRESET_FILTER="${row#*$'\t'}"
}
