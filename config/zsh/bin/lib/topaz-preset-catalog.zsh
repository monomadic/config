#!/usr/bin/env zsh

# Shared Topaz preset catalog for wrappers in this directory.
# Row format:
#   picker<TAB>display_label<TAB>preset_name<TAB>preset_flag<TAB>filter_complex<TAB>output_ext<TAB>video_args<TAB>metadata

topaz_preset_catalog_rows() {
  emulate -L zsh

  print -r -- $'encode\tVJ 1080p-ish to 4K60 ProRes Proxy\tVJ 1080p-ish to 4K60 ProRes Proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ 4K60 ProRes Proxy. Proteus keeps graphics clean while Apollo creates 60fps motion for projection and live visual workflows.'
  print -r -- $'encode\tVJ mixed 720p/random to 4K ProRes Proxy\tVJ mixed 720p/random to 4K ProRes Proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.03:noise=0.18:details=0.48:halo=-0.05:blur=0.28:compression=0.30:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ mixed-source 4K ProRes Proxy. Stronger compression cleanup and deblur for 720p or irregular graphics sources.'
  print -r -- $'encode\tHQ 1080p to clean sharp 4K HEVC\tHQ 1080p to clean sharp 4K HEVC\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.06:details=0.42:halo=-0.03:blur=0.10:compression=0.08:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 40M\tvideoai=HQ 1080p to 4K. Light Proteus enhancement for already-good sources: restrained denoise, compression repair, and detail recovery.'
  print -r -- $'encode\tLow-light iPhone 1080p to 4K 35Mbps HEVC\tLow-light iPhone 1080p to 4K 35Mbps HEVC\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.05:noise=0.45:details=0.32:halo=-0.04:blur=0.24:compression=0.28:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 35M\tvideoai=Low-light iPhone 1080p to 4K. Proteus denoise and moderate recovery, preserving a little grain to avoid waxy faces.'
  print -r -- $'encode\tOvercompressed 4K cleanup HEVC\tOvercompressed 4K cleanup HEVC\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0.02:noise=0.20:details=0.22:halo=-0.08:blur=0.16:compression=0.62:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tmp4\t-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 35M\tvideoai=Overcompressed 4K cleanup. Same-size Proteus pass prioritizing compression repair, reduced halos, and restrained detail recovery.'

  print -r -- $'encode\t4K Enhance - clean digital look without cartoonish output\t4K Enhance - clean digital look without cartoonish output\t--filter_complex\tscale=1920:-1,tvai_up=model=prob-4:scale=2:preblur=0:noise=0.10:details=0.30:halo=0.02:blur=0.06:compression=0.10:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tProb-4 2x - medium enhancement\tProb-4 2x - medium enhancement\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.35:details=0.60:halo=0:blur=0.20:compression=0.30:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tProb-4 2x - strong detail recovery\tProb-4 2x - strong detail recovery\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.55:details=0.90:halo=0:blur=0.50:compression=0.45:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\t1080p to 4K - balanced upscale\t1080p to 4K - balanced upscale\t--filter_complex\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.08:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\t1080p to 4K - Proteus 2x sharp detail\t1080p to 4K - Proteus 2x sharp detail\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0.08:noise=0.22:details=1:halo=-0.06:blur=1:compression=1:estimate=8:grain=0.05:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0'
  print -r -- $'encode\tFocus Fix - mild recovery\tFocus Fix - mild recovery\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.15:details=0.55:halo=0.04:blur=0.18:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tFocus Fix - stronger recovery\tFocus Fix - stronger recovery\t--filter_complex\ttvai_up=model=prob-4:scale=1:preblur=0:noise=0.22:details=0.80:halo=0.08:blur=0.28:compression=0.18:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\t30fps to 60fps - Apollo / interpolation\t30fps to 60fps - Apollo / interpolation\t--filter_complex\ttvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
  print -r -- $'encode\t24fps to 60fps - Apollo / interpolation\t24fps to 60fps - Apollo / interpolation\t--filter_complex\ttvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tIris MQ - moderate cleanup and sharpening\tIris MQ - moderate cleanup and sharpening\t--filter_complex\ttvai_up=model=iris-mq:scale=1:preblur=0:noise=0.25:details=0.50:halo=0:blur=0.15:compression=0.20:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tArtemis HQ - gentle enhancement for decent sources\tArtemis HQ - gentle enhancement for decent sources\t--filter_complex\ttvai_up=model=ahq-13:scale=1:preblur=0:noise=0.15:details=0.35:halo=0:blur=0.10:compression=0.15:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tNyx - strong denoise for dark/noisy footage\tNyx - strong denoise for dark/noisy footage\t--filter_complex\ttvai_up=model=nyx-3:scale=1:preblur=0:noise=0.80:details=0.25:halo=0:blur=0.20:compression=0.25:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1'
  print -r -- $'encode\tProteus manual - balanced recover + denoise\tProteus manual - balanced recover + denoise\t--filter_complex\ttvai_up=model=prob-3:scale=1:preblur=0:noise=0.40:details=0.55:halo=0.05:blur=0.15:compression=0.25:estimate=8:grain=0.02:gsize=2:device=0:vram=0.95:instances=1'

  print -r -- $'simple\tVJ 1080p-ish -> 4K60 ProRes Proxy\tvj-1080p-ish-to-4k60-prores-proxy\t--filter_complex\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tmov\t-c:v prores_ks -profile:v proxy -pix_fmt yuv422p10le -vendor apl0\tvideoai=VJ 4K60 ProRes Proxy. Proteus keeps graphics clean while Apollo creates 60fps motion for projection and live visual workflows.'
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
  print -r -- $'[Interpolate] Apollo 60fps\tInterpolate\tapollo-interpolate-60fps\ttvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo 60fps. Motion-only Apollo pass to 60fps. Slower, higher-quality interpolation for natural motion; inspect hands, hair, cuts, flashes, and fast pans for warping.'
  print -r -- $'[Interpolate] Chronos Fast 60fps\tInterpolate\tchronos-fast-interpolate-60fps\ttvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Chronos Fast 60fps. Motion-only Chronos Fast pass to 60fps. Faster and usually lighter than Apollo; useful for previews or simpler motion, with more risk on complex motion.'
  print -r -- $'[Upscale] Proteus 2x, Sharpen\tUpscale\tproteus-upscale-2x-sharpen\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.10:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale] Proteus 2x, Sharpen. 2x Proteus upscale: light denoise, medium detail recovery, mild sharpening, light compression cleanup, tiny grain. Conservative default for decent 1080p sources.'
  print -r -- $'[Upscale] Proteus 4K, Grain, Sharpen\tUpscale\tproteus-4k-grain-sharpen\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tvideoai=[Upscale] Proteus 4K, Grain, Sharpen. Forced 3840x2160 Proteus upscale: light denoise, medium detail recovery, medium sharpening/deblur, light compression cleanup, light dehalo, subtle grain. Good polished 4K target without going full rescue.'
  print -r -- $'[Upscale] Starlight Mini 2x\tUpscale\tstarlight-mini-upscale-2x\ttvai_up=model=slm-1:scale=2:device=0:vram=0.95:instances=1\tvideoai=[Upscale] Starlight Mini 2x. 2x Starlight Mini upscale with model defaults. Try on small, soft, or low-detail sources where Proteus feels too manual; compare carefully for texture changes.'
  print -r -- $'[Upscale, Enhance] Proteus 4K Compression Fix\tUpscale, Enhance\tproteus-4k-compression-fix\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=-0.51:noise=-0.79:details=1:halo=-0.75:blur=1:compression=1:prenoise=0:estimate=8:grain=0.78:gsize=2:parameters=grain_type=gaussian\\\\:grain_sigma=0.73:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tvideoai=[Upscale, Enhance] Proteus 4K Compression Fix. Forced 3840x2160 Proteus rescue pass: max compression repair, max detail, max sharpening, negative denoise/dehalo/antialias-deblur, heavy gaussian grain. For damaged stream copies; aggressive and likely stylized on decent sources.'
  print -r -- $'[Upscale, Interpolate] Proteus 2x, Chronos Fast 60fps\tUpscale, Interpolate\tproteus-upscale-2x-sharpen-chronos-fast-60fps\ttvai_up=model=prob-4:scale=2:preblur=0:noise=0.18:details=0.35:halo=0.02:blur=0.10:compression=0.12:estimate=8:grain=0.01:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Upscale, Interpolate] Proteus 2x, Chronos Fast 60fps. 2x Proteus plus Chronos Fast 60fps: conservative upscale, light cleanup, mild sharpening, tiny grain, faster interpolation. Good quick 1080p30 to 4K-ish/60fps workflow.'
  print -r -- $'[Upscale, Interpolate] Proteus 4K, Apollo 60fps\tUpscale, Interpolate\tproteus-4k-apollo-60fps\ttvai_up=model=prob-4:scale=0:w=3840:h=2160:preblur=0:noise=0.10:details=0.38:halo=-0.04:blur=0.18:compression=0.20:estimate=8:grain=0.03:gsize=2:device=0:vram=0.95:instances=1,tvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1,scale=w=3840:h=2160:flags=lanczos:threads=0\tvideoai=[Upscale, Interpolate] Proteus 4K, Apollo 60fps. Forced 3840x2160 Proteus plus Apollo 60fps: polished 4K upscale, light denoise, medium detail/sharpening, subtle grain, higher-quality motion interpolation. Best all-in-one quality path when motion preview looks clean.'
}

# Enhancement preset rows for the mpv interactive render menu.
# Presets are resolution-agnostic: each filter body carries an @SCALE@ token that the
# Lua substitutes from the Output tab's resolution choice (scale=1 / scale=2 /
# scale=0:w=..:h=.. plus a lanczos tail for the forced-4K form). `scales` declares
# which resolutions the model supports: a comma list drawn from {1, 2, 4k}.
# Row format:
#   category<TAB>display<TAB>slug<TAB>scales<TAB>filter_body<TAB>blurb<TAB>metadata
# category: detail | repair | sharpen | focus-fix
# blurb: short one-line description shown as the preset's second menu line.
topaz_enhancement_preset_rows() {
  emulate -L zsh

  # Presets are placed along independent axes (detail invention, compression repair,
  # sharpening/deblur, grain, model choice) instead of a single intensity ladder, so
  # they stay visually distinct. All manual Proteus rows use estimate=0 (absolute /
  # manual mode): sliders are fixed 0-1 values, not offsets from a per-clip auto
  # estimate, so a preset renders the same look on every source. In this mode `halo`
  # is dehalo strength (higher = fewer halos) and `blur` is sharpen/deblur — the
  # main source of crunch and ringing. Non-Proteus models run on their own character
  # (scale only). All model codes verified present in the bundle.

  # --- Detail (generative / reconstruction models and Proteus detail recipes) ---
  print -r -- $'detail\tStarlight Mini\tstarlight-mini\t2\ttvai_up=model=slm-1:@SCALE@:device=0:vram=0.95:instances=1\tGenerative; strongest true detail invention (2x only)\tvideoai=[Detail] Starlight Mini. Diffusion-style reconstruction — the strongest genuine detail generation here, and it fails differently from Proteus (texture/face hallucination rather than halos). Compare carefully on faces and text.'
  print -r -- $'detail\tGaia HQ — Animation\tgaia-hq\t2\ttvai_up=model=ghq-5:@SCALE@:device=0:vram=0.95:instances=1\tClean, smooth; best for CG / animation (2x only)\tvideoai=[Detail] Gaia HQ. Clean, smooth, low-noise upscale. Softer and more poster-like than Proteus; best for graphics, animation, CG and already-clean footage.'
  print -r -- $'detail\tIris — Faces\tiris-faces\t1,2,4k\ttvai_up=model=iris-3:@SCALE@:device=0:vram=0.95:instances=1\tFace & compressed-source model; rebuilds skin/eyes\tvideoai=[Detail] Iris. Tuned for faces and compressed/streamed sources — reconstructs skin, eyes and hair. Very different texture from Proteus; watch for facial hallucination.'
  print -r -- $'detail\tProteus — Detail Max\tproteus-detail-max\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0:noise=0.08:details=1:halo=0.15:blur=0.12:compression=0.85:estimate=0:grain=0.03:gsize=2:device=0:vram=0.95:instances=1\tMax invention + compression fix, minimal sharpen\tvideoai=[Detail] Proteus Detail Max. Manual Proteus: maxed detail generation and heavy compression repair with sharpening kept low and dehalo engaged — the aggressive invented-detail look without the crunch and halos.'
  print -r -- $'detail\tProteus — Max + Regrain\tproteus-max-regrain\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0:noise=0.20:details=0.90:halo=0.10:blur=0.10:compression=0.75:estimate=0:grain=0.09:gsize=2.5:device=0:vram=0.95:instances=1\tHigh detail under film grain; hides ringing + plastic\tvideoai=[Detail] Proteus Max + Regrain. High detail generation and strong compression repair finished with a visible grain layer — the grain reads as fine texture and masks any residual ringing or plastic smoothing. Filmic alternative to Detail Max.'

  # --- Repair (compression / noise repair) ---
  print -r -- $'repair\tProteus — Max Compression Repair\tproteus-max-compression-repair\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0.05:noise=0.25:details=0.40:halo=0.25:blur=0.08:compression=1:estimate=0:grain=0.02:gsize=2:device=0:vram=0.95:instances=1\tCompression repair maxed; keeps moderate detail\tvideoai=[Repair] Proteus Max Compression Repair. Compression repair at maximum and strong dehalo, but moderate detail recovery kept in so blocking and mosquito noise clean up without going waxy-soft.'
  print -r -- $'repair\tNyx — Heavy Denoise\tnyx-heavy-denoise\t1\ttvai_up=model=nyx-3:@SCALE@:device=0:vram=0.95:instances=1\tHeavy denoise for dark / grainy footage\tvideoai=[Repair] Nyx Heavy Denoise. Dedicated denoise model for dark/grainy/high-ISO footage. Very smooth output; can wax skin and erase fine texture — use where noise is the main problem.'
  print -r -- $'repair\tProteus — Clean + Regrain\tproteus-clean-regrain\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0:noise=0.40:details=0.30:halo=0.15:blur=0.12:compression=0.70:estimate=0:grain=0.08:gsize=2.5:device=0:vram=0.95:instances=1\tDeep clean, then re-grain to hide smoothing\tvideoai=[Repair] Proteus Clean + Regrain. Strong compression/noise cleanup then a re-grain layer to hide the smoothing — restores a filmic texture on waxy or over-cleaned sources.'

  # --- Sharpen & Texture (micro-detail and stylized texture passes) ---
  print -r -- $'sharpen\tTheia — Fine Detail\ttheia-fine-detail\t1\ttvai_up=model=thd-3:@SCALE@:device=0:vram=0.95:instances=1\tMicro-detail sharpen; less plastic than Proteus\tvideoai=[Sharpen] Theia Detail. Fine micro-detail/sharpen model. Crisp textures and edges with a different character (and less plastic look) than Proteus sharpening.'
  print -r -- $'sharpen\tProteus — Heavy Grain Texture\tproteus-heavy-grain-texture\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0:noise=0.10:details=0.50:halo=0.10:blur=0.20:compression=0.30:estimate=0:grain=0.14:gsize=3:device=0:vram=0.95:instances=1\tStrong film-grain look; stylized texture\tvideoai=[Sharpen] Proteus Heavy Grain Texture. Moderate enhancement under a strong film-grain layer (gsize 3) — a deliberately stylized, filmic texture pass that masks smoothing and generates perceived detail.'

  # --- Focus Fix (deblur rescue ladder; Proteus-based, so any resolution works —
  # --- deblur artifacts do magnify when upscaled, judge the preview carefully) ---
  print -r -- $'focus-fix\tProteus Deblur — Light\tproteus-deblur-light\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0.10:noise=0.15:details=0.55:halo=0.20:blur=0.60:compression=0.30:estimate=0:grain=0.02:gsize=2:device=0:vram=0.95:instances=1\tModerate deblur for slightly soft footage\tvideoai=[Focus Fix] Proteus Deblur Light. Manual Proteus led by firm sharpening/deblur with moderate detail recovery and dehalo raised to compensate. For soft or slightly out-of-focus sources; will crunch footage that is already sharp.'
  print -r -- $'focus-fix\tProteus Deblur — Strong\tproteus-deblur-strong\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0.15:noise=0.18:details=0.80:halo=0.20:blur=0.80:compression=0.20:estimate=0:grain=0.01:gsize=2:device=0:vram=0.95:instances=1\tHeavy deblur for soft / missed-focus footage\tvideoai=[Focus Fix] Proteus Deblur Strong. Heavy sharpening/deblur with high detail recovery and dehalo raised to fight the resulting halos. For clearly soft or slightly missed-focus shots; expect edge artifacts on sharp sources.'
  print -r -- $'focus-fix\tProteus Deblur — Max Rescue\tproteus-deblur-max-rescue\t1,2,4k\ttvai_up=model=prob-4:@SCALE@:preblur=0.35:noise=0.10:details=1:halo=0.30:blur=1:compression=0.50:estimate=0:grain=0:gsize=2:device=0:vram=0.95:instances=1\tLast-resort rescue; everything maxed, stylized\tvideoai=[Focus Fix] Proteus Deblur Max Rescue. Deblur, detail and anti-alias all maxed with dehalo at its strongest — a last-resort rescue for badly out-of-focus or motion-blurred footage. Heavily stylized; expect ringing and crunch.'
}

# Advanced insight rows for the mpv preset-details companion sheet (toggled with
# `d` in the render menu). Keyed by enhancement preset slug — one blob of preset-level
# knowledge per preset (no per-parameter annotations). Row kinds:
#   <slug>\tstrategy\t<text>               — why the recipe works (top callout)
#   <slug>\twatch\t<text>                  — the failure mode to inspect for
#   <slug>\tvs\t<other slug>\t<text>       — when to pick this vs its neighbour
topaz_preset_insights() {
  emulate -L zsh

  print -r -- $'__original__\tstrategy\tThe control. Every preset is judged against this frame — flick Space often to keep your eye honest before trusting a render.'

  print -r -- $'starlight-mini\tstrategy\tDiffusion-style reconstruction: it re-imagines pixels rather than filtering them, so it generates texture that Proteus can only sharpen toward. 2x only — on 1080p that is the full 4K target.'
  print -r -- $'starlight-mini\twatch\tHallucination — faces, text and logos can change identity, not just clarity. Inspect anything a viewer would recognise.'
  print -r -- $'starlight-mini\tvs\tproteus-detail-max\tDetail Max amplifies what exists; Starlight invents. When it wins it wins big, but it is slower and less predictable.'

  print -r -- $'gaia-hq\tstrategy\tTrained on CG and animation: preserves line art and flat fills without inventing photographic texture. On graphics, invention is a bug — this is the anti-hallucination pick.'
  print -r -- $'gaia-hq\twatch\tLive footage comes out soft and poster-like — edges survive but skin and fabric flatten.'
  print -r -- $'gaia-hq\tvs\tstarlight-mini\tDrawn or rendered source: Gaia. Photographic source: Starlight.'

  print -r -- $'proteus-detail-max\tstrategy\tThe thesis preset: invention and compression repair near max, sharpening almost off. Detail comes from the model generating structure, not from edge contrast — so it stays clean where crunchy presets ring. Works at any output size; same-size passes are subtle at fit-to-window zoom, so judge at 100%.'
  print -r -- $'proteus-detail-max\twatch\tOn pristine sources max invention can over-texture flat areas (skin, sky). If surfaces look busy, this is why.'
  print -r -- $'proteus-detail-max\tvs\tproteus-max-regrain\tSame idea finished with grain. Pick Max + Regrain when this reads too clean or plastic.'

  print -r -- $'proteus-max-regrain\tstrategy\tGrain is your friend twice over: it reads as invented fine texture, and it masks the ringing and plastic smoothing that heavy enhancement leaves behind. gsize 2.5 is coarse enough to read as film rather than sensor noise.'
  print -r -- $'proteus-max-regrain\twatch\tGrain is baked in — it cannot be removed later, and HEVC spends real bitrate encoding it.'
  print -r -- $'proteus-max-regrain\tvs\tproteus-detail-max\tTrades peak invention for texture that hides the plastic look. Pick this when Detail Max reads too clean.'

  print -r -- $'iris-faces\tstrategy\tA face-trained network: rebuilds eyes, skin and hair from priors about what faces look like — priors that survive compression heavy enough to destroy generic detail.'
  print -r -- $'iris-faces\twatch\tThe same priors are the risk: a face can drift toward a generic face rather than the real one. Check identity on people you know.'
  print -r -- $'iris-faces\tvs\tstarlight-mini\tBoth hallucinate; Iris is specialised. Faces prominent: Iris. Everything else: Starlight.'

  print -r -- $'proteus-max-compression-repair\tstrategy\tCompression repair at 1.0 with detail held moderate: kills blocking and mosquito noise without the waxy look of a pure repair pass. Artifacts read as detail to other presets — clean first.'
  print -r -- $'proteus-max-compression-repair\twatch\tAt maximum repair some legitimate fine texture goes with the artifacts — check hair, fabric and foliage.'
  print -r -- $'proteus-max-compression-repair\tvs\tproteus-detail-max\tDetail Max already carries 0.85 repair. Reach for this only when artifacts, not detail, are the problem.'

  print -r -- $'nyx-heavy-denoise\tstrategy\tA dedicated denoiser, not an enhancer — noise modelling that the generic Proteus noise slider cannot match on high-ISO or low-light footage.'
  print -r -- $'nyx-heavy-denoise\twatch\tWaxes skin and erases fine texture — whatever it cannot distinguish from noise is gone for good.'
  print -r -- $'nyx-heavy-denoise\tvs\tproteus-clean-regrain\tNyx removes; Clean + Regrain removes then re-textures. If the result must still look like film, pick that.'

  print -r -- $'proteus-clean-regrain\tstrategy\tTwo moves in one pass: scrub compression and noise hard, then lay uniform grain over the scrubbed surface. The grain unifies patches the cleanup smoothed unevenly — the classic restoration trick. gsize 2.5 matches Max + Regrain, so cleaned and upscaled clips cut together.'
  print -r -- $'proteus-clean-regrain\twatch\tIf damage varies shot to shot the cleanup varies too, and the grain hides it unevenly.'
  print -r -- $'proteus-clean-regrain\tvs\tproteus-max-compression-repair\tSame cleanup instinct; Max Compression Repair keeps the honest scrubbed look, this one dresses it.'

  print -r -- $'theia-fine-detail\tstrategy\tA micro-contrast model: enhances existing fine structure rather than generating new texture, so it reads crisp without the AI-plastic signature.'
  print -r -- $'theia-fine-detail\twatch\tIt cannot create — on genuinely soft or damaged sources there is nothing for it to work with.'
  print -r -- $'theia-fine-detail\tvs\tproteus-detail-max\tGeneration vs enhancement: a good source that wants bite takes Theia; missing texture takes Detail Max.'

  print -r -- $'proteus-heavy-grain-texture\tstrategy\tA look, not a fix: moderate enhancement buried under heavy gsize-3 grain. The grain does aesthetic work — film emulation — as much as masking.'
  print -r -- $'proteus-heavy-grain-texture\twatch\tExpensive to encode and impossible to undo — grade before this pass, not after.'
  print -r -- $'proteus-heavy-grain-texture\tvs\tproteus-max-regrain\tMax + Regrain uses grain as seasoning; this uses it as the dish.'

  print -r -- $'proteus-deblur-light\tstrategy\tLed by the sharpen slider the other presets suppress. Right when softness is the defect — missed focus, generational loss — because no amount of detail generation fixes an unsharp edge. Upscaling works but magnifies the edge work — prefer Original unless the preview holds up.'
  print -r -- $'proteus-deblur-light\twatch\tOn already-sharp footage this is pure crunch — it can look worse than the source.'
  print -r -- $'proteus-deblur-light\tvs\tproteus-deblur-strong\tStart here. Step up to Strong only when edges are still visibly soft.'

  print -r -- $'proteus-deblur-strong\tstrategy\tThe rescue for soft-but-usable footage: heavy deblur and high detail, with dehalo raised to cancel the ringing the sharpening creates.'
  print -r -- $'proteus-deblur-strong\twatch\tCheck high-contrast edges — rooflines, text, speculars — at 100%; that is where ringing shows first.'
  print -r -- $'proteus-deblur-strong\tvs\tproteus-deblur-max-rescue\tTry this first. Max Rescue is the same idea with nothing held back, at triple the artifacts.'

  print -r -- $'proteus-deblur-max-rescue\tstrategy\tEverything at the stops: deblur, detail and anti-alias maxed, dehalo at its ceiling fighting the fallout. For footage you would otherwise delete — stylised-sharp beats honestly-unusable.'
  print -r -- $'proteus-deblur-max-rescue\twatch\tEdges gain a drawn, illustrated quality. If this still is not enough, the shot is gone.'
  print -r -- $'proteus-deblur-max-rescue\tvs\tproteus-deblur-strong\tIf Strong got you 80% of the way, stop — this buys the last 20% at triple the artifacts.'
}

# Interpolation preset rows for the mpv interactive workflow (frame-rate stage).
# The Lua injects a "None" option ahead of these motion models.
# Models verified in this Topaz bundle: apo-8 (Apollo v8), apf-2 (Apollo Fast),
# chr-2 (Chronos), chf-3 (Chronos Fast). NOTE: the bare "apollo" alias does NOT
# exist here and crashes tvai_fi — always use the versioned codes.
# Row format:
#   display<TAB>slug<TAB>fi_filter<TAB>metadata
topaz_interpolation_preset_rows() {
  emulate -L zsh

  print -r -- $'Apollo 60fps — best quality\tapollo-interpolate-60fps\ttvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo 60fps. Highest-quality motion; best at detecting and replacing duplicate/repeated frames (jerky 30fps exports, stutter). Inspect hands, hair, cuts, fast pans.'
  print -r -- $'Apollo Fast 60fps — quick pass\tapollo-fast-interpolate-60fps\ttvai_fi=model=apf-2:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo Fast 60fps. Lighter, much faster Apollo variant. Good default when Apollo full is too slow and motion is simple-to-moderate.'
  print -r -- $'Chronos 60fps — complex motion\tchronos-interpolate-60fps\ttvai_fi=model=chr-2:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Chronos 60fps. Alternative motion engine; often handles complex/organic motion (crowds, water, particles) with fewer warp artifacts than Apollo. Try when Apollo smears.'
  print -r -- $'Chronos Fast 60fps — fastest\tchronos-fast-interpolate-60fps\ttvai_fi=model=chf-3:slowmo=1:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Chronos Fast 60fps. Fastest option; use for previews or bulk conversions of simple motion. More risk on complex motion.'
  print -r -- $'Apollo 120fps — high-Hz / slow-mo\tapollo-interpolate-120fps\ttvai_fi=model=apo-8:slowmo=1:rdt=0.01:fps=120:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo 120fps. Generate 120fps for high-refresh displays or as raw material to conform to slow motion later. Large files; motion errors are 4x more visible.'
  print -r -- $'Apollo Slow-mo 2x\tapollo-slowmo-2x\ttvai_fi=model=apo-8:slowmo=2:rdt=0.01:fps=60:device=0:vram=0.95:instances=1\tvideoai=[Interpolate] Apollo Slow-mo 2x. Halves playback speed while synthesizing in-between frames at 60fps output — turns 30/60fps clips into smooth half-speed slow motion. Audio will desync (video-only effect).'
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
