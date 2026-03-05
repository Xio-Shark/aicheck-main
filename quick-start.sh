#!/usr/bin/env sh
set -eu

if ! command -v cargo >/dev/null 2>&1; then
  echo "未检测到 cargo，请先安装 Rust 工具链。"
  exit 1
fi

cargo run -p aidoc-cli -- explain
