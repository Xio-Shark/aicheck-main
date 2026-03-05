# aidoc

[![CI](https://github.com/Xio-Shark/aicheck-main/actions/workflows/ci.yml/badge.svg)](https://github.com/Xio-Shark/aicheck-main/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Xio-Shark/aicheck-main)](https://github.com/Xio-Shark/aicheck-main/releases)

只读诊断工具（Rust Workspace），用于把构建/安装类报错整理为可交接的结构化报告。

## 特性

- ✅ 只读安全：不修改系统配置、注册表、环境变量
- ✅ 多平台支持：Windows / Linux / macOS (x64 & ARM64)
- ✅ 结构化输出：Markdown / JSON 格式
- ✅ 智能脱敏：自动隐藏敏感信息
- ✅ 签名匹配：识别常见错误模式
- ✅ Docker 支持：一键容器化运行
- ✅ LLM 集成：可选 AI 摘要功能

## 快速安装

### 从 GitHub Release 下载

```bash
# Linux
wget https://github.com/Xio-Shark/aicheck-main/releases/latest/download/aidoc-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf aidoc-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
./aidoc --help

# macOS
curl -LO https://github.com/Xio-Shark/aicheck-main/releases/latest/download/aidoc-v1.0.0-x86_64-apple-darwin.tar.gz
tar -xzf aidoc-v1.0.0-x86_64-apple-darwin.tar.gz
./aidoc --help

# Windows
# 下载 aidoc-v1.0.0-x86_64-pc-windows-msvc.zip 并解压
```

### 从源码构建

#### 1) 安装 Rust

Windows：
```powershell
.\install-rust.ps1
```

Linux/macOS：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 2) 构建项目

```bash
cargo build --release -p aidoc-cli
./target/release/aidoc --help
```

## 使用示例

### 基础诊断

```bash
# 诊断当前环境
aidoc diagnose --format md

# 只读边界说明
aidoc explain

# 错误签名匹配
echo "bash: pip: command not found" | aidoc paste --format json
```

### Docker 运行

```bash
# 快速启动
docker compose up -d
docker compose exec aidoc aidoc diagnose

# 清理
docker compose down
```

## 项目结构

```text
crates/
  aidoc-cli        # 命令行入口
  aidoc-core       # 核心模型与pack拼装
  aidoc-signatures # 签名规则与命中逻辑
  aidoc-sandbox    # 只读白名单执行器
  aidoc-probes     # 环境探针采集
  aidoc-redact     # 脱敏
  aidoc-output     # md/json 渲染
  aidoc-llm        # 可选 LLM 摘要
```

## 开发

### 运行测试

```bash
cargo test --all --locked
```

### 代码覆盖率

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 性能基准测试

```bash
cargo bench
```

### 安全审计

```bash
cargo install cargo-audit
cargo audit
```

## LLM 功能（可选）

```bash
cargo run -p aidoc-cli --features llm -- diagnose --llm on --llm-provider ollama
```

需要设置 `AIDOC_API_KEY` 环境变量。

## 退出码

- `0`：诊断完成且无问题
- `1`：诊断完成且发现问题
- `2`：工具自身错误
- `3`：权限不足
- `4`：LLM 调用失败

## 文档

- [使用指南](USAGE_GUIDE.md)
- [安装说明](INSTALL_RUST_ZH.md)
- [发布流程](RELEASE.md)
- [更新日志](CHANGELOG.md)

## 许可证

MIT License
