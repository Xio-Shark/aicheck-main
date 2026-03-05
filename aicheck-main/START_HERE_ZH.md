# START HERE (ZH)

## 1. 安装 Rust

Windows:

```powershell
.\install-rust.ps1
```

Linux/macOS:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## 2. 快速验证

Windows:

```powershell
.\quick-start.ps1
```

Linux/macOS:

```bash
bash quick-start.sh
```

## 3. 运行测试

```bash
cargo test --all --locked
```

## 4. Docker 一键启动

Windows:

```powershell
.\up.ps1
```

Linux/macOS:

```bash
bash up.sh
```

或使用 Makefile：

```bash
cp .env.example .env
make up
```
