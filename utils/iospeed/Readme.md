# iospeed

A tiny disk I/O benchmark with a realtime TUI, built with
[bubbletea](https://github.com/charmbracelet/bubbletea). Writes a test file
of a given size, reads it back, and shows live throughput while each phase
runs — the thing `dd`-based shell scripts are bad at.

```
  iospeed  ·  disk I/O benchmark

  target /Volumes/Scratch   size 1 GiB   block 4 MiB   cache bypass on

  write   ███████████████████████░░░░░░░░░░░  67%  1.84 GB/s  eta 1.2s
  read    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%  waiting

  q to quit
```

## Usage

```sh
iospeed                      # 1 GiB test file in the current directory
iospeed /Volumes/SomeDisk    # benchmark another volume
iospeed -size 4G -block 8M   # bigger file, bigger blocks
iospeed -keep                # leave the test file behind
iospeed -cached              # allow the page cache (measures RAM, not disk)
```

Piping output (or running from a script) skips the TUI and prints plain
results.

## Notes

- On macOS the page cache is bypassed with `F_NOCACHE`, so numbers reflect
  the actual disk. Without this, the read phase would just measure RAM.
- Write timing includes a final `fsync`.
- Speeds are decimal (MB/s, GB/s) like most disk benchmarks; sizes are
  binary (MiB, GiB).
- The test file (`.iospeed-<pid>.tmp`) is removed afterwards, including on
  ctrl+c.

## Install

```sh
./build.zsh    # builds and installs to ~/.local/bin
```
