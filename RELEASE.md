# Release Checklist

本文档提供 aidoc 项目的发布检查清单和流程说明。

## 发布前检查清单

在创建新版本之前，请确保完成以下检查：

### 1. 代码质量

- [ ] 所有测试通过（`cargo test`）
- [ ] 代码通过 lint 检查（`cargo clippy`）
- [ ] 代码格式化正确（`cargo fmt --check`）
- [ ] 只读验证通过（`make verify-readonly`，如果有）

### 2. 版本号更新

- [ ] 更新 `Cargo.toml` 中的版本号
- [ ] 版本号遵循语义化版本规范（Semantic Versioning）
  - 主版本号：不兼容的 API 变更
  - 次版本号：向后兼容的功能新增
  - 修订号：向后兼容的问题修复

### 3. 文档更新

- [ ] 更新 `README.md`（如有新功能或变更）
- [ ] 更新 `CHANGELOG.md`（记录本次发布的变更）
- [ ] 检查所有文档链接有效

### 4. 依赖检查

- [ ] 检查依赖是否有安全漏洞（`cargo audit`）
- [ ] 更新过时的依赖（如需要）

### 5. GitHub Secrets 配置

确保 GitHub 仓库已配置以下 Secrets：

- [ ] `NPM_TOKEN`：npm 发布令牌
  - 登录 [npmjs.com](https://www.npmjs.com/)
  - 进入 Account Settings → Access Tokens
  - 创建 Automation token
  - 在 GitHub 仓库的 Settings → Secrets and variables → Actions 中添加

## 发布流程

### 步骤 1：准备发布

```bash
# 1. 确保在主分支且代码最新
git checkout main
git pull origin main

# 2. 运行所有测试
cargo test --all

# 3. 构建检查
cargo build --release -p aidoc-cli

# 4. 本地测试 npm 包安装
cd npm
export AIDOC_BINARY_PATH=../target/release/aidoc
npm install
./bin/aidoc.js --help
cd ..
```

### 步骤 2：更新版本号

```bash
# 编辑 Cargo.toml，更新 version 字段
# 例如：version = "0.1.1"

# 提交版本更新
git add Cargo.toml
git commit -m "chore: bump version to 0.1.1"
git push origin main
```

### 步骤 3：创建并推送标签

```bash
# 创建版本标签（注意 v 前缀）
git tag v0.1.1

# 推送标签到 GitHub（这将触发发布工作流）
git push origin v0.1.1
```

### 步骤 4：监控发布流程

1. 访问 GitHub Actions 页面：`https://github.com/YOUR_ORG/aidoc/actions`
2. 查看 "Release" 工作流的执行状态
3. 检查以下步骤是否成功：
   - ✅ 版本验证
   - ✅ 4 个平台的构建（Linux, macOS x64, macOS ARM64, Windows）
   - ✅ GitHub Release 创建
   - ✅ npm 包发布

### 步骤 5：验证发布

#### 验证 GitHub Release

1. 访问 `https://github.com/YOUR_ORG/aidoc/releases`
2. 确认新版本的 Release 已创建
3. 检查是否包含 4 个二进制压缩包：
   - `aidoc-v0.1.1-x86_64-unknown-linux-gnu.tar.gz`
   - `aidoc-v0.1.1-x86_64-apple-darwin.tar.gz`
   - `aidoc-v0.1.1-aarch64-apple-darwin.tar.gz`
   - `aidoc-v0.1.1-x86_64-pc-windows-msvc.zip`

#### 验证 npm 发布

```bash
# 检查 npm 包版本
npm view @aidoc/cli version

# 在新目录测试安装
mkdir test-install
cd test-install
npm install -g @aidoc/cli

# 测试命令
aidoc --help

# 清理
npm uninstall -g @aidoc/cli
cd ..
rm -rf test-install
```

## 发布失败处理

### 构建失败

如果某个平台的构建失败：

1. 查看 GitHub Actions 日志，定位错误原因
2. 修复代码问题
3. 删除失败的标签：
   ```bash
   git tag -d v0.1.1
   git push origin :refs/tags/v0.1.1
   ```
4. 重新执行发布流程

### GitHub Release 创建失败

如果 Release 创建失败但构建成功：

1. 手动从 GitHub Actions artifacts 下载构建产物
2. 在 GitHub 网页上手动创建 Release
3. 上传下载的压缩包

### npm 发布失败

如果 npm 发布失败：

1. 检查 `NPM_TOKEN` 是否有效
2. 检查包名是否已被占用
3. 如果 GitHub Release 已创建，可以手动发布 npm 包：
   ```bash
   cd npm
   npm version 0.1.1 --no-git-tag-version
   npm publish --access public
   ```

## 回滚发布

### 回滚 npm 包（24 小时内）

```bash
# 取消发布（仅在发布后 24 小时内有效）
npm unpublish @aidoc/cli@0.1.1

# 或者弃用版本
npm deprecate @aidoc/cli@0.1.1 "This version has critical bugs, please upgrade"
```

### 删除 GitHub Release

1. 访问 `https://github.com/YOUR_ORG/aidoc/releases`
2. 找到要删除的 Release
3. 点击 "Delete" 按钮
4. 删除对应的 Git 标签：
   ```bash
   git tag -d v0.1.1
   git push origin :refs/tags/v0.1.1
   ```

## 发布后任务

- [ ] 在 GitHub Release 中添加发布说明（Release Notes）
- [ ] 更新项目文档网站（如有）
- [ ] 在社交媒体或社区宣布新版本（如需要）
- [ ] 监控用户反馈和问题报告

## 紧急修复发布

对于紧急 bug 修复：

1. 创建修复分支：`git checkout -b hotfix/v0.1.2`
2. 修复问题并测试
3. 更新版本号（修订号 +1）
4. 合并到主分支
5. 按照正常发布流程创建标签

## 常见问题

### Q: 如何发布预发布版本（alpha, beta, rc）？

A: 使用预发布版本号格式：
```bash
# 更新 Cargo.toml: version = "0.2.0-beta.1"
git tag v0.2.0-beta.1
git push origin v0.2.0-beta.1

# npm 发布时使用 tag
cd npm
npm publish --tag beta
```

### Q: 如何测试发布流程而不实际发布？

A: 在测试仓库中：
1. Fork 项目到个人账号
2. 配置测试用的 npm 包名（如 `@yourname/aidoc-test`）
3. 在 fork 仓库中执行完整发布流程
4. 验证所有步骤正常工作

### Q: 构建时间过长怎么办？

A: GitHub Actions 的 Rust 缓存应该能显著加速构建。如果仍然很慢：
- 检查是否启用了 `Swatinem/rust-cache`
- 考虑减少并行构建的平台数量
- 使用 GitHub Actions 的 self-hosted runners

## 相关资源

- [Semantic Versioning](https://semver.org/)
- [npm Publishing Guide](https://docs.npmjs.com/cli/v9/commands/npm-publish)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo Book - Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
