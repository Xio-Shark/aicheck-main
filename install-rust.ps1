$ErrorActionPreference = "Stop"

if (Get-Command cargo -ErrorAction SilentlyContinue) {
  cargo --version
  Write-Host "已检测到 Rust 工具链。"
  exit 0
}

$installer = Join-Path $env:TEMP "rustup-init.exe"
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $installer
Start-Process -FilePath $installer -ArgumentList "-y" -Wait

Write-Host "Rust 安装完成，请关闭并重新打开终端后继续。"
