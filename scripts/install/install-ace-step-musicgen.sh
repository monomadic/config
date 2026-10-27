#!/usr/bin/env bash
# install or update ACE-Step 1.5 (local music generation)
# https://github.com/ace-step/ACE-Step-1.5   (MIT)
#
# Usage:
#   ace-step.sh            install (or update if already cloned)
#   ace-step.sh --test     also run the repo's quick_test.sh after install
#   ace-step.sh --no-sync  skip `uv sync` (just pull + wrappers)
#
# Env overrides:
#   ACESTEP_DIR   clone location   (default: ~/opt/ace-step)
#   BIN_DIR       wrapper location (default: ~/.local/bin)
#   ACESTEP_PORT  gradio port      (default: 7860)
#   ACESTEP_API_PORT api port      (default: 8001)
set -euo pipefail

ACESTEP_DIR="${ACESTEP_DIR:-$HOME/opt/ace-step}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
ACESTEP_PORT="${ACESTEP_PORT:-7860}"
ACESTEP_API_PORT="${ACESTEP_API_PORT:-8001}"
REPO="https://github.com/ace-step/ACE-Step-1.5.git"

RUN_TEST=0; DO_SYNC=1
for a in "$@"; do
  case "$a" in
    --test) RUN_TEST=1 ;;
    --no-sync) DO_SYNC=0 ;;
    -h|--help) sed -n '2,16p' "$0"; exit 0 ;;
    *) echo "unknown arg: $a" >&2; exit 2 ;;
  esac
done

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

# --- preflight -------------------------------------------------------------
[[ "$(uname -s)" == "Darwin" ]] || log "not macOS — continuing, but MLX launch scripts won't apply"
if [[ "$(uname -m)" != "arm64" ]]; then
  log "warning: Intel Mac — expect CPU-only inference (slow)"
fi
command -v git >/dev/null || die "git not found"

if ! command -v uv >/dev/null; then
  log "installing uv"
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
  command -v uv >/dev/null || die "uv install failed; open a new shell and re-run"
fi
log "uv $(uv --version | awk '{print $2}')"

# --- clone / update --------------------------------------------------------
if [[ -d "$ACESTEP_DIR/.git" ]]; then
  log "updating $ACESTEP_DIR"
  git -C "$ACESTEP_DIR" fetch --quiet origin
  git -C "$ACESTEP_DIR" pull --ff-only --quiet
else
  log "cloning to $ACESTEP_DIR"
  mkdir -p "$(dirname "$ACESTEP_DIR")"
  git clone --depth 1 "$REPO" "$ACESTEP_DIR"
fi
cd "$ACESTEP_DIR"
log "at $(git rev-parse --short HEAD)"

# --- deps ------------------------------------------------------------------
# pyproject pins python >=3.11,<3.12 (as of 2026-10); uv handles the interpreter.
if (( DO_SYNC )); then
  log "uv sync (first run pulls torch ~2GB)"
  uv sync
fi

# --- .env (survives repo updates) -----------------------------------------
if [[ ! -f .env ]]; then
  log "creating .env from .env.example"
  cp .env.example .env
  # Apple Silicon defaults: 2B turbo DiT, lightweight 0.6B LM, pt backend.
  # Bump to acestep-5Hz-lm-1.7B on >=32GB unified memory; XL needs ~20GB free.
  {
    echo ""
    echo "# --- set by install/ace-step.sh ---"
    echo "ACESTEP_CONFIG_PATH=acestep-v15-turbo"
    echo "ACESTEP_LM_MODEL_PATH=acestep-5Hz-lm-0.6B"
    echo "PORT=${ACESTEP_PORT}"
    echo "LANGUAGE=en"
  } >> .env
else
  log ".env exists, leaving it alone"
fi

# --- wrappers --------------------------------------------------------------
mkdir -p "$BIN_DIR"
chmod +x start_gradio_ui_macos.sh start_api_server_macos.sh 2>/dev/null || true

cat > "$BIN_DIR/acestep" <<EOF
#!/usr/bin/env bash
# Gradio UI (MLX on Apple Silicon). Models auto-download on first run (~5-10GB).
cd "$ACESTEP_DIR" && exec ./start_gradio_ui_macos.sh "\$@"
EOF

cat > "$BIN_DIR/acestep-api" <<EOF
#!/usr/bin/env bash
# REST API server on :${ACESTEP_API_PORT}  (docs/en/API.md)
cd "$ACESTEP_DIR" && exec ./start_api_server_macos.sh "\$@"
EOF

cat > "$BIN_DIR/acestep-raw" <<EOF
#!/usr/bin/env bash
# Direct uv entrypoint, bypassing the macOS launch script (PyTorch/MPS path).
# e.g. acestep-raw --server-name 0.0.0.0   |   acestep-raw --help
cd "$ACESTEP_DIR" && exec uv run acestep "\$@"
EOF

chmod +x "$BIN_DIR"/acestep "$BIN_DIR"/acestep-api "$BIN_DIR"/acestep-raw
log "wrappers: $BIN_DIR/{acestep,acestep-api,acestep-raw}"
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) log "note: $BIN_DIR not in PATH" ;; esac

# --- optional smoke test ---------------------------------------------------
if (( RUN_TEST )); then
  log "running quick_test.sh (downloads models)"
  chmod +x quick_test.sh && ./quick_test.sh
fi

cat <<EOF

done.
  acestep        → http://localhost:${ACESTEP_PORT}
  acestep-api    → http://localhost:${ACESTEP_API_PORT}
  config         → $ACESTEP_DIR/.env
  update         → $(basename "$0")   (re-run; pulls + resyncs)
  MPS OOM?       → lower duration / batch, or set ACESTEP_LM_MODEL_PATH= (empty) for DiT-only
EOF
