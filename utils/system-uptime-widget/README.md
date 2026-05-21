# System Uptime Widget

Simple macOS menu bar app that shows system uptime in a compact format.

## Build

```bash
go build -ldflags="-s -w" .
```

## Output

The menu bar title rounds to the nearest useful unit:

```text
30m
5h
2d
```
