#!/usr/bin/env node

const { spawnSync } = require("child_process");
const path = require("path");

// Determine binary name based on platform
const binaryName = process.platform === "win32" ? "aidoc.exe" : "aidoc";

// Construct path to the actual binary
const binaryPath = path.join(__dirname, binaryName);

// Execute the binary with all arguments forwarded
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
});

// Forward the exit code
process.exit(result.status ?? 1);
