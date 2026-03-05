#!/usr/bin/env node

const https = require("https");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");
const os = require("os");

const PACKAGE_NAME = "@aidoc/cli";
const BINARY_NAME_BASE = "aidoc";

// Platform mapping: maps Node.js platform-arch to Rust target triple
const targetMap = {
  "linux-x64": {
    target: "x86_64-unknown-linux-gnu",
    ext: "tar.gz",
    binaryName: "aidoc",
  },
  "darwin-x64": {
    target: "x86_64-apple-darwin",
    ext: "tar.gz",
    binaryName: "aidoc",
  },
  "darwin-arm64": {
    target: "aarch64-apple-darwin",
    ext: "tar.gz",
    binaryName: "aidoc",
  },
  "win32-x64": {
    target: "x86_64-pc-windows-msvc",
    ext: "zip",
    binaryName: "aidoc.exe",
  },
};

function log(message) {
  console.log(`[aidoc postinstall] ${message}`);
}

function fail(message) {
  console.error(`[aidoc postinstall] ${message}`);
  process.exit(1);
}

/**
 * Download a file from a URL to a destination path
 * @param {string} url - The URL to download from
 * @param {string} destinationPath - The local file path to save to
 * @returns {Promise<void>}
 */
function download(url, destinationPath) {
  return new Promise((resolve, reject) => {
    log(`Downloading from ${url}`);
    
    const file = fs.createWriteStream(destinationPath);
    
    const request = https.get(url, {
      headers: {
        "User-Agent": `${PACKAGE_NAME} postinstall`,
      },
    }, (response) => {
      // Handle redirects
      if (response.statusCode === 301 || response.statusCode === 302 || 
          response.statusCode === 307 || response.statusCode === 308) {
        const redirectUrl = response.headers.location;
        log(`Following redirect to ${redirectUrl}`);
        file.close();
        fs.unlinkSync(destinationPath);
        download(redirectUrl, destinationPath).then(resolve).catch(reject);
        return;
      }
      
      if (response.statusCode !== 200) {
        file.close();
        fs.unlinkSync(destinationPath);
        reject(new Error(`Download failed: status=${response.statusCode} url=${url}`));
        return;
      }
      
      response.pipe(file);
      
      file.on("finish", () => {
        file.close();
        resolve();
      });
    });
    
    request.on("error", (err) => {
      file.close();
      fs.unlinkSync(destinationPath);
      reject(err);
    });
    
    request.setTimeout(30000, () => {
      request.destroy(new Error(`Download timeout after 30s: ${url}`));
    });
  });
}

/**
 * Extract an archive file
 * @param {string} archivePath - Path to the archive file
 * @param {string} target - The Rust target triple (for logging)
 * @param {string} outputPath - Expected path of the extracted binary
 */
function extractArchive(archivePath, target, outputPath) {
  log(`Extracting to ${path.dirname(outputPath)}`);
  
  const binDir = path.dirname(outputPath);
  const ext = path.extname(archivePath);
  
  let command, args;
  
  if (ext === ".gz") {
    // Unix: tar -xzf
    command = "tar";
    args = ["-xzf", archivePath, "-C", binDir];
  } else if (ext === ".zip") {
    // Windows: PowerShell Expand-Archive
    command = "powershell";
    args = [
      "-Command",
      `Expand-Archive -Path '${archivePath}' -DestinationPath '${binDir}' -Force`,
    ];
  } else {
    fail(`Unsupported archive format: ${ext}`);
  }
  
  const result = spawnSync(command, args, { stdio: "inherit" });
  
  if (result.error) {
    fail(`Extract failed: ${result.error.message}`);
  }
  
  if (result.status !== 0) {
    fail(`${command} exited with status ${result.status}`);
  }
  
  // Verify binary exists after extraction
  if (!fs.existsSync(outputPath)) {
    fail(`Binary not found after install: ${outputPath}`);
  }
  
  log(`Extraction complete`);
}

/**
 * Copy binary from local path (for development)
 * @param {string} localPath - Path to local binary
 * @param {string} outputPath - Destination path
 */
function copyFromLocalBinary(localPath, outputPath) {
  log(`Using local binary from ${localPath}`);
  
  if (!fs.existsSync(localPath)) {
    fail(`AIDOC_BINARY_PATH not found: ${localPath}`);
  }
  
  const binDir = path.dirname(outputPath);
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }
  
  fs.copyFileSync(localPath, outputPath);
  log(`Copied local binary to ${outputPath}`);
}

/**
 * Main installation function
 */
async function main() {
  // Detect platform
  const platformKey = `${process.platform}-${process.arch}`;
  const target = targetMap[platformKey];
  
  if (!target) {
    fail(
      `Unsupported platform: ${platformKey}. Supported platforms: ${Object.keys(targetMap).join(", ")}`
    );
  }
  
  const binDir = path.join(__dirname, "bin");
  const outputPath = path.join(binDir, target.binaryName);
  
  // Ensure bin directory exists
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }
  
  // Check for local binary path (development mode)
  const localBinaryPath = process.env.AIDOC_BINARY_PATH;
  if (localBinaryPath) {
    copyFromLocalBinary(localBinaryPath, outputPath);
    
    // Set executable permission on Unix
    if (process.platform !== "win32") {
      fs.chmodSync(outputPath, 0o755);
    }
    
    log("Installation complete");
    return;
  }
  
  // Read package.json to get version
  const packageJsonPath = path.join(__dirname, "package.json");
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
  const version = packageJson.version;
  const tag = `v${version}`;
  
  // Construct download URL
  const baseUrl = process.env.AIDOC_RELEASE_BASE_URL || 
    "https://github.com/aidoc/aidoc/releases/download";
  const archiveName = `aidoc-${tag}-${target.target}.${target.ext}`;
  const archiveUrl = `${baseUrl}/${tag}/${archiveName}`;
  
  // Download archive
  const archivePath = path.join(os.tmpdir(), archiveName);
  
  try {
    await download(archiveUrl, archivePath);
  } catch (error) {
    fail(`Download failed: ${error.message}`);
  }
  
  // Extract archive
  try {
    extractArchive(archivePath, target.target, outputPath);
  } catch (error) {
    fail(`Extraction failed: ${error.message}`);
  } finally {
    // Clean up archive file
    if (fs.existsSync(archivePath)) {
      fs.unlinkSync(archivePath);
    }
  }
  
  // Set executable permission on Unix
  if (process.platform !== "win32") {
    fs.chmodSync(outputPath, 0o755);
    log(`Set executable permission on ${outputPath}`);
  }
  
  log("Installation complete");
}

// Run main function
main().catch((error) => {
  fail(`Unexpected error: ${error.message}`);
});
