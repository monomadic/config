# VirtualDJ stem file formats

The canonical, evidence-labeled reference lives in the VirtualDJ API
reference project:

    ~/src/virtualdj-api-reference/docs/Stem File Format.md

Quick summary for tool authors in this repo (`config/zsh/bin/vdjstems-*`):

- **Sidecar `.vdjstems`** (what `vdjstems make` produces by default):
  Matroska, exactly 5 AAC-LC stereo streams titled
  `vocal, hihat, bass, instruments, kick`, no mixed track, named
  `<original filename incl. ext>.vdjstems` next to the original.
- **Standalone stems**: 6-track M4A (`mixed track` first + the 5 stems),
  MP4Box udta track names, brands `isom:512/mp42/mp41`, flat storage,
  named `.m4a` — **never** name an MP4 `.vdjstems`.
- 4-stem sources: duplicate drums into kick + hihat at **−6.0206 dB each**;
  never count the duplicate twice in a synthesized master.

Update the reference project first; keep this stub short.
