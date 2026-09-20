#!/usr/bin/env node

const path = require("path");
const { spawnSync } = require("child_process");

const PLATFORM_PACKAGES = {
  "linux-x64":     "@rivolink/leaf-linux-x64",
  "linux-arm64":   "@rivolink/leaf-linux-arm64",
  "darwin-x64":    "@rivolink/leaf-darwin-x64",
  "darwin-arm64":  "@rivolink/leaf-darwin-arm64",
  "win32-x64":     "@rivolink/leaf-win32-x64",
  "android-arm64": "@rivolink/leaf-android-arm64",
};

const key = `${process.platform}-${process.arch}`;
const pkgName = PLATFORM_PACKAGES[key];

if (!pkgName) {
  console.error(`[leaf] Unsupported platform: ${key}`);
  process.exit(1);
}

let binaryPath;
try {
  const isWindows = process.platform === "win32";
  const binaryName = isWindows ? "leaf.exe" : "leaf";
  binaryPath = require.resolve(path.join(pkgName, binaryName));
} catch {
  console.error(`[leaf] Binary package not found for ${key}.`);
  console.error(`[leaf] Reinstall: npm install -g @rivolink/leaf`);
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
  env: {
    ...process.env,
    LEAF_CURRENT_EXE: binaryPath,
  },
});

process.exit(result.status ?? 1);
