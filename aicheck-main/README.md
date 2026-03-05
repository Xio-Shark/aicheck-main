# aidoc

只读诊断工具（Rust Workspace），用于把构建/安装类报错整理为可交接的结构化报告。

## 快速开始

### 1) 安装 Rust

Windows：

```powershell
.\install-rust.ps1
```

Linux/macOS：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2) 本地运行

Windows：

```powershell
.\quick-start.ps1
```

Linux/macOS：

```bash
bash quick-start.sh
```

### 3) 运行测试

```bash
cargo test --all --locked
```

## Docker 复现

Windows：

```powershell
.\up.ps1
```

Linux/macOS：

```bash
bash up.sh
```

手动命令：

```bash
docker compose up -d --build
docker compose ps
docker compose exec -T aidoc aidoc explain
docker compose down --remove-orphans
```

## 常用命令

```bash
# 只读边界说明
cargo run -p aidoc-cli -- explain

# 诊断当前环境
cargo run -p aidoc-cli -- diagnose --format md

# 对原始错误做签名匹配与脱敏
printf "bash: pip: command not found\n" | cargo run -p aidoc-cli -- paste --format md

# 合并日志与快照
{
  printf "bash: pip: command not found\n"
  printf "\n---AIDOC-SECTION-BREAK---\n"
  printf '{"os":"linux","arch":"x86_64","shell":"bash","elevated":false,"path_preview":[],"toolchains":[],"proxy":{"http_proxy":null,"https_proxy":null,"no_proxy":null},"network":[]}\n'
} | cargo run -p aidoc-cli -- pack --format md
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

## 只读与安全约束

- 不写系统配置、不改注册表、不改环境变量。
- 外部命令通过白名单执行器并带超时。
- 敏感信息先脱敏再进入后续流程。
- 密钥仅从环境变量读取（如 `AIDOC_API_KEY`）。

## LLM（可选）

```bash
cargo run -p aidoc-cli --features llm -- diagnose --llm on --llm-provider ollama --llm-dry-run
```

- 优先级：CLI 参数 > 环境变量 > 配置文件
- 远程 provider 需要 `AIDOC_API_KEY`

## 文档

- [START_HERE_ZH.md](START_HERE_ZH.md)
- [USAGE_GUIDE.md](USAGE_GUIDE.md)
- [INSTALL_RUST_ZH.md](INSTALL_RUST_ZH.md)
- [RELEASE.md](RELEASE.md)
- [CHANGELOG.md](CHANGELOG.md)

## 退出码

- `0`：诊断完成且无问题
- `1`：诊断完成且发现问题
- `2`：工具自身错误
- `3`：权限不足
- `4`：LLM 调用失败或未启用 LLM 构建
