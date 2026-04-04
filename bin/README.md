# Bin Layout

`bin/` is for active user-facing commands.

Rules:

- Keep commands here if you expect to call them directly from your shell.
- If a command is just a stable shim around a retained binary, keep the shim here and store the real binary in `vendor/bin/`.
- Do not keep the same command name in both `bin/` and `config/zsh/bin/`; choose one home and keep wrappers explicit.
- If a file is a demo, experiment, or reference example, move it to `archive/examples/`.
- If a file is machine output or logs, do not keep it here.

Current intent:

- `bin/`: active commands
- `vendor/bin/`: retained compiled/custom binaries
- `archive/examples/bin/`: demos and reference scripts
