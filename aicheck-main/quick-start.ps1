$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Error "未检测到 cargo。请先执行 .\install-rust.ps1 安装 Rust 工具链。"
  exit 1
}

cargo run -p aidoc-cli -- explain
