# ============================================================================
# Audio Stem Separation
# ============================================================================

# Stem splitting lives in ~/.zsh/bin/vdjstems-split (MLX RoFormer pipeline);
# the old MVSEP-MDX23 function it replaced is in git history (downloads.zsh,
# pre-July-2026).

# Print the duration of every WAV in the CWD — quick sanity check that a
# stem set is aligned before packing (mismatched lengths → drifting stems).
vdjstems-check-wav-lengths() {
  local f dur
  for f in *.wav(N); do
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
