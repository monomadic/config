# VirtualDJ stem file formats

The canonical, evidence-labeled reference lives in the VirtualDJ API
reference project:

    ~/src/virtualdj-api-reference/docs/Stem File Format.md

Quick summary for tool authors in this repo (`config/zsh/bin/vdjstems-*`):

- **Sidecar `.vdjstems`** (what `vdjstems make` produces by default):
  Matroska, exactly 5 AAC-LC stereo streams titled
  `vocal, hihat, bass, instruments, kick`, no mixed track, named
  `<original filename incl. ext>.vdjstems` next to the original.
  **Sample rate must match the original file** (VDJ plays a mismatch
  pitch-shifted and stuttery); cap >96 kHz originals at 44.1 kHz.
  **Matroska `writing-application` must be stamped
  `VirtualDJ <version>.stems2`** (mkvpropedit) — VDJ silently ignores
  unstamped sidecars (A/B verified 2026-07-18).
  Codec is free: AAC is VDJ's own choice, but FLAC (16- and 24-bit),
  ALAC, and PCM sidecars are all verified working
  (`vdjstems-pack --sidecar -l` emits FLAC). The stamp check is
  version-tolerant (a 2025 stamp passes) and titles are case-insensitive.
- ALAC **standalone** works but must be **16-bit (s16p) on every stream**
  — an s32p master plays stuttery/pitch-broken (VDJ's MP4 ALAC is 16-bit
  only). FLAC-in-MP4 standalones also verified (needs ffmpeg `-f mp4
  -strict experimental`; the `.m4a`/ipod muxer refuses FLAC).
- **Standalone stems**: 6-track M4A (`mixed track` first + the 5 stems),
  MP4Box udta track names, brands `isom:512/mp42/mp41`, flat storage,
  named `.m4a` — **never** name an MP4 `.vdjstems`.
- 4-stem sources: duplicate drums into kick + hihat at **−6.0206 dB each**;
  never count the duplicate twice in a synthesized master.
- Standalone master = **plain unity-gain sum of the 5 stems**
  (`amix normalize=0`): VDJ sums stems at unity when they're active, so an
  attenuated master jumps ~12 dB on stem mute/unmute.

Update the reference project first; keep this stub short.
