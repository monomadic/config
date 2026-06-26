#!/usr/bin/env zsh

# Shared Topaz preset catalog for wrappers in this directory.
# Row format:
#   picker<TAB>display_label<TAB>preset_name<TAB>preset_flag<TAB>filter_complex<TAB>output_ext<TAB>video_args<TAB>metadata

topaz_preset_catalog_rows() {
  emulate -L zsh

  print -r -- $'run\tVJ 1080p-ish to 4K60 ProRes Proxy\tVJ 1080p-ish to 4K60 ProRes Proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ 4K60 ProRes Proxy. Proteus keeps graphics clean while Apollo creates 60fps motion for projection and live visual workflows.'
  print -r -- $'run\tVJ mixed 720p/random to 4K ProRes Proxy\tVJ mixed 720p/random to 4K ProRes Proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.03:noise=0.18:details=0.48:halo=-0.05:blur=0.28:compression=0.30:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ mixed-source 4K ProRes Proxy. Stronger compression cleanup and deblur for 720p or irregular graphics sources.'
  print -r -- $'run\tHQ 1080p to clean sharp 4K HEVC\tHQ 1080p to clean sharp 4K HEVC\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.06:details=0.42:halo=-0.03:blur=0.10:compression=0.08:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 40M\tvideoai=HQ 1080p to 4K. Light Proteus enhancement for already-good sources: restrained denoise, compression repair, and detail recovery.'
  print -r -- $'run\tLow-light iPhone 1080p to 4K 35Mbps HEVC\tLow-light iPhone 1080p to 4K 35Mbps HEVC\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.05:noise=0.45:details=0.32:halo=-0.04:blur=0.24:compression=0.28:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 35M\tvideoai=Low-light iPhone 1080p to 4K. Proteus denoise and moderate recovery, preserving a little grain to avoid waxy faces.'
  print -r -- $'run\tOvercompressed 4K cleanup HEVC\tOvercompressed 4K cleanup HEVC\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0.02:noise=0.20:details=0.22:halo=-0.08:blur=0.16:compression=0.62:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 35M\tvideoai=Overcompressed 4K cleanup. Same-size Proteus pass prioritizing compression repair, reduced halos, and restrained detail recovery.'

  print -r -- $'run\t4K Enhance - clean digital look without cartoonish output\t4K Enhance - clean digital look without cartoonish output\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.10:details=0.30:halo=0.02:blur=0.06:compression=0.10:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tProb-4 2x - medium enhancement\tProb-4 2x - medium enhancement\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.35:details=0.60:halo=0:blur=0.20:compression=0.30:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tProb-4 2x - strong detail recovery\tProb-4 2x - strong detail recovery\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.55:details=0.90:halo=0:blur=0.50:compression=0.45:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\t1080p to 4K - balanced upscale\t1080p to 4K - balanced upscale\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.08:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\t1080p to 4K - Proteus 2x sharp detail\t1080p to 4K - Proteus 2x sharp detail\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.08:noise=0.22:details=1:halo=-0.06:blur=1:compression=1:estimate=8:grain=0.05:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0'
  print -r -- $'run\tFocus Fix - mild recovery\tFocus Fix - mild recovery\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.15:details=0.55:halo=0.04:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tFocus Fix - stronger recovery\tFocus Fix - stronger recovery\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.22:details=0.80:halo=0.08:blur=0.28:compression=0.18:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\t30fps to 60fps - Apollo / interpolation\t30fps to 60fps - Apollo / interpolation\t--filter_complex\ttvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
  print -r -- $'run\t24fps to 60fps - Apollo / interpolation\t24fps to 60fps - Apollo / interpolation\t--filter_complex\ttvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
  print -r -- $'run\tIris MQ - moderate cleanup and sharpening\tIris MQ - moderate cleanup and sharpening\t--filter_complex\ttvai_up=model=iris-mq:scale=1:preblur=0:noise=0.25:details=0.50:halo=0:blur=0.15:compression=0.20:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tArtemis HQ - gentle enhancement for decent sources\tArtemis HQ - gentle enhancement for decent sources\t--filter_complex\ttvai_up=model=ahq-13:scale=1:preblur=0:noise=0.15:details=0.35:halo=0:blur=0.10:compression=0.15:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tNyx - strong denoise for dark/noisy footage\tNyx - strong denoise for dark/noisy footage\t--filter_complex\ttvai_up=model=nyx-3:scale=1:preblur=0:noise=0.80:details=0.25:halo=0:blur=0.20:compression=0.25:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'run\tProteus manual - balanced recover + denoise\tProteus manual - balanced recover + denoise\t--filter_complex\ttvai_up=model=prob-3:scale=1:preblur=0:noise=0.40:details=0.55:halo=0.05:blur=0.15:compression=0.25:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'

  print -r -- $'simple\tVJ 1080p-ish -> 4K60 ProRes Proxy\tvj-1080p-ish-to-4k60-prores-proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ 4K60 ProRes Proxy. Proteus keeps graphics clean while Apollo creates 60fps motion for projection and live visual workflows.'
  print -r -- $'simple\tVJ mixed 720p/random -> 4K ProRes Proxy\tvj-mixed-720p-random-to-4k-prores-proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.03:noise=0.18:details=0.48:halo=-0.05:blur=0.28:compression=0.30:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ mixed-source 4K ProRes Proxy. Stronger compression cleanup and deblur for 720p or irregular graphics sources.'
  print -r -- $'simple\tHQ 1080p -> Clean Sharp 4K HEVC\t1080p-hq-to-clean-sharp-4k-hevc\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.06:details=0.42:halo=-0.03:blur=0.10:compression=0.08:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 40M\tvideoai=HQ 1080p to 4K. Light Proteus enhancement for already-good sources: restrained denoise, compression repair, and detail recovery.'
  print -r -- $'simple\tLow-light iPhone 1080p -> 4K 35Mbps HEVC\tlow-light-iphone-1080p-to-4k-35mbps-hevc\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.05:noise=0.45:details=0.32:halo=-0.04:blur=0.24:compression=0.28:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 35M\tvideoai=Low-light iPhone 1080p to 4K. Proteus denoise and moderate recovery, preserving a little grain to avoid waxy faces.'
  print -r -- $'simple\tOvercompressed 4K Cleanup HEVC\tcompressed-4k-cleanup-hevc\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0.02:noise=0.20:details=0.22:halo=-0.08:blur=0.16:compression=0.62:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 35M\tvideoai=Overcompressed 4K cleanup. Same-size Proteus pass prioritizing compression repair, reduced halos, and restrained detail recovery.'

  print -r -- $'simple\t4K Conservative Cleanup\t4k-conservative-cleanup\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.20:details=0.22:halo=0.03:blur=0.08:compression=0.18:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Balanced Enhancement\t4k-balanced-enhancement\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.32:details=0.36:halo=0.05:blur=0.14:compression=0.30:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Aggressive Artifact Cleanup\t4k-aggressive-artifact-cleanup\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.50:details=0.28:halo=0.10:blur=0.18:compression=0.42:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Focus Fix Light\t4k-focus-fix-light\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.12:details=0.26:halo=0.02:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t4K Focus Fix Strong\t4k-focus-fix-strong\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.34:halo=0.03:blur=0.34:compression=0.16:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Clean Upscale\t1080p-to-4k-clean\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.16:details=0.26:halo=0.02:blur=0.08:compression=0.12:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Medium Enhancement\t1080p-to-4k-medium\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.28:details=0.34:halo=0.04:blur=0.12:compression=0.22:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Strong Recovery\t1080p-to-4k-strong\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.42:details=0.44:halo=0.06:blur=0.20:compression=0.30:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p -> 4K Proteus 2x Detail\t1080p-to-4k-proteus-2x-detail\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.08:noise=0.22:details=1:halo=-0.06:blur=1:compression=1:estimate=8:grain=0.05:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0'
  print -r -- $'simple\t30fps -> 60fps Interpolation\t30fps-to-60fps\t--filter_complex\ttvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
  print -r -- $'simple\t1080p30 -> 4K60\t1080p30-to-4k60\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.26:details=0.32:halo=0.03:blur=0.10:compression=0.20:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
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

# Transform rows used by the newer two-step workflow.
# Row format:
#   display_label<TAB>categories<TAB>slug<TAB>filter_complex<TAB>metadata
topaz_transform_preset_rows() {
  emulate -L zsh

  print -r -- $'[Cleanup] Proteus Compression Cleanup\tCleanup\tproteus-compression-cleanup\ttvai_up=model=prob-4:scale=1:preblur=0.02:noise=0.20:details=0.22:halo=-0.08:blur=0.16:compression=0.62:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Cleanup] Proteus Compression Cleanup. Same-size Proteus pass: heavy compression repair, light denoise, mild detail recovery, medium sharpening/deblur, stronger dehalo, tiny grain. Good for visibly blocky 4K or already-upscaled sources.'
  print -r -- $'[Denoise] Nyx Dark Footage\tDenoise\tnyx-denoise-dark-footage\ttvai_up=model=nyx-3:scale=1:preblur=0:noise=0.80:details=0.25:halo=0:blur=0.20:compression=0.25:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Denoise] Nyx Dark Footage. Same-size Nyx pass: very heavy denoise, low detail recovery, medium sharpening, light compression cleanup, no grain. Best for dark/noisy clips; can look waxy on clean skin or fine texture.'
  print -r -- $'[Enhance] Iris MQ, Sharpen\tEnhance\tiris-mq-enhance-sharpen\ttvai_up=model=iris-mq:scale=1:preblur=0:noise=0.25:details=0.50:halo=0:blur=0.15:compression=0.20:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Enhance] Iris MQ, Sharpen. Same-size Iris MQ pass: moderate denoise, strong detail recovery, light-medium sharpening, light compression cleanup, tiny grain. Useful for faces and soft detail; watch for face or texture hallucination.'
  print -r -- $'[Focus Fix] Proteus Light\tFocus Fix\tproteus-focus-fix-light\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.15:details=0.55:halo=0.04:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Focus Fix] Proteus Light. Same-size Proteus focus-style pass: light denoise, strong detail recovery, medium sharpening/deblur, light compression cleanup, tiny grain. Good first try for decent-but-soft footage.'
  print -r -- $'[Focus Fix] Proteus Strong\tFocus Fix\tproteus-focus-fix-strong\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.22:details=0.80:halo=0.08:blur=0.28:compression=0.18:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Focus Fix] Proteus Strong. Same-size Proteus focus-style pass: light-medium denoise, very strong detail recovery, heavy sharpening/deblur, light compression cleanup, tiny grain. Use when Light helps but still looks soft; higher risk of crunchy edges.'
  print -r -- $'[Interpolate] Apollo 60fps\tInterpolate\tapollo-interpolate-60fps\ttvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo 60fps. Motion-only Apollo pass to 60fps. Slower, higher-quality interpolation for natural motion; inspect hands, hair, cuts, flashes, and fast pans for warping.'
  print -r -- $'[Interpolate] Chronos Fast 60fps\tInterpolate\tchronos-fast-interpolate-60fps\ttvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Chronos Fast 60fps. Motion-only Chronos Fast pass to 60fps. Faster and usually lighter than Apollo; useful for previews or simpler motion, with more risk on complex motion.'
  print -r -- $'[Upscale] Proteus 2x, Sharpen\tUpscale\tproteus-upscale-2x-sharpen\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.10:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale] Proteus 2x, Sharpen. 2x Proteus upscale: light denoise, medium detail recovery, mild sharpening, light compression cleanup, tiny grain. Conservative default for decent 1080p sources.'
  print -r -- $'[Upscale] Proteus 4K, Grain, Sharpen\tUpscale\tproteus-4k-grain-sharpen\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tvideoai=[Upscale] Proteus 4K, Grain, Sharpen. Forced 3840x2160 Proteus upscale: light denoise, medium detail recovery, medium sharpening/deblur, light compression cleanup, light dehalo, subtle grain. Good polished 4K target without going full rescue.'
  print -r -- $'[Upscale] Starlight Mini 2x\tUpscale\tstarlight-mini-upscale-2x\ttvai_up=model=slm-1:scale=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale] Starlight Mini 2x. 2x Starlight Mini upscale with model defaults. Try on small, soft, or low-detail sources where Proteus feels too manual; compare carefully for texture changes.'
  print -r -- $'[Upscale, Enhance] Proteus 4K Compression Fix\tUpscale, Enhance\tproteus-4k-compression-fix\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=-0.51:noise=-0.79:details=1:halo=-0.75:blur=1:compression=1:prenoise=0:estimate=8:grain=0.78:gsize=2:parameters=grain_type=gaussian\\\\:grain_sigma=0.73:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tvideoai=[Upscale, Enhance] Proteus 4K Compression Fix. Forced 3840x2160 Proteus rescue pass: max compression repair, max detail, max sharpening, negative denoise/dehalo/antialias-deblur, heavy gaussian grain. For damaged stream copies; aggressive and likely stylized on decent sources.'
  print -r -- $'[Upscale, Interpolate] Proteus 2x, Chronos Fast 60fps\tUpscale, Interpolate\tproteus-upscale-2x-sharpen-chronos-fast-60fps\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.10:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Upscale, Interpolate] Proteus 2x, Chronos Fast 60fps. 2x Proteus plus Chronos Fast 60fps: conservative upscale, light cleanup, mild sharpening, tiny grain, faster interpolation. Good quick 1080p30 to 4K-ish/60fps workflow.'
  print -r -- $'[Upscale, Interpolate] Proteus 4K, Apollo 60fps\tUpscale, Interpolate\tproteus-4k-apollo-60fps\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tvideoai=[Upscale, Interpolate] Proteus 4K, Apollo 60fps. Forced 3840x2160 Proteus plus Apollo 60fps: polished 4K upscale, light denoise, medium detail/sharpening, subtle grain, higher-quality motion interpolation. Best all-in-one quality path when motion preview looks clean.'
}

# Enhancement preset rows for the mpv interactive render menu.
# The Lua groups these by category, hides the upscale categories on >=4K sources,
# and (for @4K@ rows) substitutes an orientation-aware 4K scale plus appends a matching
# lanczos tail. filter_body carries NO trailing scale tail.
# Row format:
#   category<TAB>display<TAB>slug<TAB>filter_body<TAB>metadata
# category: upscale-2x | upscale-4k | repair | sharpen | focus-fix
topaz_enhancement_preset_rows() {
  emulate -L zsh

  # --- Upscale 2x (literal scale=2; shown only for <4K sources) ---
  print -r -- $'upscale-2x\tProteus 2x, Sharpen\tproteus-upscale-2x-sharpen\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.10:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale 2x] Proteus 2x Sharpen. 2x Proteus upscale: light denoise, medium detail recovery, mild sharpening, light compression cleanup, tiny grain. Conservative default for decent sources.'
  print -r -- $'upscale-2x\tStarlight Mini 2x\tstarlight-mini-upscale-2x\ttvai_up=model=slm-1:scale=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale 2x] Starlight Mini 2x. 2x Starlight Mini with model defaults. Try on small/soft/low-detail sources where Proteus feels too manual; compare for texture changes.'

  # --- Upscale to 4K (forced orientation-aware 4K via @4K@; shown only for <4K sources) ---
  print -r -- $'upscale-4k\tProteus 4K, Sharpen\tproteus-4k-sharpen\ttvai_up=model=prob-4:@4K@:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.10:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale to 4K] Proteus 4K Sharpen. Forced 4K Proteus upscale: light denoise, medium detail recovery, mild sharpening, light compression cleanup, tiny grain.'
  print -r -- $'upscale-4k\tProteus 4K, Grain, Sharpen\tproteus-4k-grain-sharpen\ttvai_up=model=prob-4:@4K@:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale to 4K] Proteus 4K Grain Sharpen. Forced 4K Proteus upscale: light denoise, medium detail/sharpening, light compression cleanup, light dehalo, subtle grain. Polished 4K target.'

  # --- Repair (same-size compression / noise repair) ---
  print -r -- $'repair\tProteus Compression Cleanup\tproteus-compression-cleanup\ttvai_up=model=prob-4:scale=1:preblur=0.02:noise=0.20:details=0.22:halo=-0.08:blur=0.16:compression=0.62:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Repair] Proteus Compression Cleanup. Same-size Proteus pass: heavy compression repair, light denoise, mild detail recovery, stronger dehalo. Good for blocky or already-upscaled footage.'
  print -r -- $'repair\tNyx Dark Denoise\tnyx-denoise-dark-footage\ttvai_up=model=nyx-3:scale=1:preblur=0:noise=0.80:details=0.25:halo=0:blur=0.20:compression=0.25:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Repair] Nyx Dark Denoise. Same-size Nyx pass: very heavy denoise, low detail recovery, medium sharpening. Best for dark/noisy clips; can look waxy on clean skin.'

  # --- Sharpen (detail recovery / sharpening emphasis) ---
  print -r -- $'sharpen\tIris MQ, Sharpen\tiris-mq-enhance-sharpen\ttvai_up=model=iris-mq:scale=1:preblur=0:noise=0.25:details=0.50:halo=0:blur=0.15:compression=0.20:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Sharpen] Iris MQ Sharpen. Same-size Iris MQ pass: moderate denoise, strong detail recovery, light-medium sharpening. Good for faces and soft detail; watch for hallucination.'
  print -r -- $'sharpen\tProteus Sharpen\tproteus-sharpen\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.20:details=0.55:halo=0.04:blur=0.30:compression=0.15:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Sharpen] Proteus Sharpen. Same-size Proteus pass: light denoise, strong detail recovery, heavy sharpening/deblur. Crisper edges on soft-but-clean sources.'

  # --- Focus Fix (recover soft / slightly out-of-focus footage) ---
  print -r -- $'focus-fix\tProteus Light\tproteus-focus-fix-light\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.15:details=0.55:halo=0.04:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Focus Fix] Proteus Light. Same-size focus-style pass: light denoise, strong detail recovery, medium sharpening/deblur. Good first try for soft footage.'
  print -r -- $'focus-fix\tProteus Strong\tproteus-focus-fix-strong\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.22:details=0.80:halo=0.08:blur=0.28:compression=0.18:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Focus Fix] Proteus Strong. Same-size focus-style pass: very strong detail recovery, heavy sharpening/deblur. Use when Light still looks soft; higher risk of crunchy edges.'
}

# Interpolation preset rows for the mpv interactive workflow (frame-rate stage).
# The Lua injects a "None" option ahead of these motion models.
# Row format:
#   display<TAB>slug<TAB>fi_filter<TAB>metadata
topaz_interpolation_preset_rows() {
  emulate -L zsh

  print -r -- $'Apollo 60fps\tapollo-interpolate-60fps\ttvai_fi=model=apollo:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo 60fps. Higher-quality motion; best at detecting and replacing duplicate/repeated frames. Inspect hands, hair, cuts, fast pans.'
  print -r -- $'Chronos Fast 60fps\tchronos-fast-interpolate-60fps\ttvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Chronos Fast 60fps. Faster and usually lighter than Apollo; good for simpler motion, more risk on complex motion.'
}

# Output profile rows used by the newer two-step workflow.
# Row format:
#   display_label<TAB>slug<TAB>output_ext<TAB>video_args
topaz_output_profile_rows() {
  emulate -L zsh

  print -r -- $'HEVC constant bitrate 40mbps\thevc-cbr-40mbps\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 40M -constant_bit_rate 1'
  print -r -- $'HEVC variable bitrate\thevc-vbr\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -q:v 65 -spatial_aq 1'
  print -r -- $'ProRes 422 Proxy\tprores-422-proxy\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0'
}
