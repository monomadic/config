#!/bin/sh
# Build native RIFE video CLI; install executable and SwiftPM model/shader bundles together.
set -eu
cd "$(dirname "$0")/../../src/utils/rife-video"
./prepare.sh
swift build -c release
libexec="$HOME/.local/libexec/rife-video"
mkdir -p "$libexec" "$HOME/.local/bin"
install -m 755 .build/release/rife-video "$libexec/rife-video"
for bundle in .build/release/RifeMetal_*.bundle; do
  ditto "$bundle" "$libexec/$(basename "$bundle")"
done
# SwiftPM resolves Bundle.module beside the executable, so keep the bundles there.
# A tiny launcher also works when ~/.local/bin is reached via a symlink.
cat > "$HOME/.local/bin/rife-video" <<'SH'
#!/bin/sh
exec "$HOME/.local/libexec/rife-video/rife-video" "$@"
SH
chmod 755 "$HOME/.local/bin/rife-video"
echo "installed: $HOME/.local/bin/rife-video"
