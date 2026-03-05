# Rust 安装指南（中文）

## Windows

1. 打开 PowerShell（建议管理员权限）。
2. 在项目根目录执行：

```powershell
.\install-rust.ps1
```

3. 关闭并重新打开终端。
4. 验证：

```powershell
cargo --version
rustc --version
```

## Linux/macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo --version
rustc --version
```

## 常见问题

### 终端提示找不到 cargo

重开终端，或手动加载环境变量：

```bash
source ~/.cargo/env
```
