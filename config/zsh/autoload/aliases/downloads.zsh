# ============================================================================
# Audio Stem Separation
# ============================================================================

stem-mdx23() {
  local input_file="$1"
  local basename=$(basename "$input_file")
  local output_dir="$HOME/Music/Stems/$basename"

  mkdir -p "$output_dir"

  cd "$HOME/Music/Stems/MVSEP-MDX23-Colab_v2.1" || { print -u2 "missing MVSEP-MDX23 dir"; return 1; }
  source .venv/bin/activate &&
    time python inference_2.2_b1.5.1_voc_ft.py \
      --input_audio "$input_file" \
      --output_folder "$output_dir" \
      --large_gpu \
      --chunk_size 500000

  cd "$output_dir" || return 1

  # Rename files
  mv *vocals.wav vocals.wav 2>/dev/null
  mv *drums.wav drums.wav 2>/dev/null
  mv *bass.wav bass.wav 2>/dev/null
  mv *other.wav other.wav 2>/dev/null
  mv *instrum.wav instrumental.wav 2>/dev/null
  rm -f *instrum2.wav
}

vdjstems-check-wav-lengths() {
  for f in kick.wav other.wav vocals.wav bass.wav hihat.wav mixed.wav; do
    dur=$(ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$f")
    printf "%s: %s\n" "$f" "$dur"
  done
}

# ============================================================================
# Download & Media Aliases
# ============================================================================

alias dp="dl-porn"

alias d=download-video
alias dmv="download-video music-video"
alias dl-youtube="download-video youtube"
alias dlu="download-video-url"
alias faphouse="download-video-faphouse"
alias dl-beatport=beatportdl-darwin-arm64
alias dl-apple-music=apple-music-dl

alias url="yt-url"

alias N_m3u8DL-RE="/Users/nom/config/bin/N_m3u8DL-RE_v0.5.1-beta_osx-arm64_20251029"
alias .dl-N_m3u8DL-RE=N_m3u8DL-RE

alias .network-quality="networkQuality -v"


# ============================================================================
# Stem Separation Aliases
# ============================================================================

alias .stem-split="demucs -d mps -n htdemucs --flac -o stems_output"
alias .stem-split-vocals="demucs -d mps -n htdemucs --flac -o stems_output --two-stems=vocals"
alias vdjstems-split-mdx23=stem-mdx23
