# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in `leaf`, please report it responsibly.

**Do not open a public issue for security vulnerabilities.**

Instead, use one of these channels:

- [GitHub private vulnerability reporting](https://github.com/RivoLink/leaf/security/advisories/new)
- Email the maintainer directly at [rivo.link@gmail.com](mailto:rivo.link@gmail.com)

Please include:

- a clear description of the issue;
- steps to reproduce;
- potential impact;
- affected versions or release assets;
- a suggested fix, if you have one.

We will review the report and respond as quickly as possible.

## Scope

This policy covers the `leaf` repository, including:

- the `leaf` CLI/TUI application;
- Markdown parsing and rendering logic;
- release workflows and published binaries;
- install scripts for Unix-like systems and Windows;
- repository documentation when it affects security-sensitive behavior.

## Best Practices for Users

- Install `leaf` from official releases or the documented install scripts.
- Keep `leaf` updated to the latest release.
- Review scripts before piping them into a shell if your environment requires stricter controls.
- On Windows, install the latest supported Microsoft Visual C++ Redistributable from Microsoft if required by the published binary.
- Avoid running untrusted Markdown content with unrealistic expectations of isolation; `leaf` is a local preview tool, not a sandbox.

## Supported Versions

Security fixes are generally applied to the latest released version.

If a vulnerability affects older releases, fixes may be backported at maintainer discretion.
