# 发布和打包 enman

本文档描述了如何构建、打包和发布 enman 的不同版本。

## 构建发布版本

### Windows 平台

使用 PowerShell 构建发布版本：

```powershell
# 构建发布版本的二进制文件
cargo build --release

# 使用打包脚本
.\scripts\build-release.ps1
```

### Linux/macOS 平台

```bash
# 构建发布版本的二进制文件
cargo build --release

# 使用打包脚本
chmod +x ./scripts/build-release.sh
./scripts/build-release.sh
```

## 打包格式

构建脚本会生成以下文件：

- `releases/enman-vX.X.X-platform-arch/` - 包含所有必要文件的目录
- `releases/enman-vX.X.X-platform-arch.zip` 或 `tar.gz` - 打包的发布档案

## 发布到 crates.io

要将 enman 发布到 crates.io，请按照以下步骤操作：

```bash
# 登录到 crates.io
cargo login <your-api-token>

# 验证包
cargo package

# 发布
cargo publish
```

## 交叉编译到其他平台

如果你需要为其他平台构建，可以使用 `cross` 工具：

```bash
# 安装 cross 工具
cargo install cross

# 为特定平台构建
cross build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-apple-darwin --release
```

## CI/CD 集成

对于自动化发布，可以使用 GitHub Actions：

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
      - name: Build
        run: |
          cargo build --release
          mkdir dist
          cp target/release/enman dist/
          cp target/release/em dist/
          cp README.md dist/
          cp LICENSE dist/
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/*
          draft: false
```

## Docker 镜像

还可以创建 Docker 镜像来分发 enman：

```dockerfile
FROM alpine:latest

RUN apk add --no-cache ca-certificates
COPY target/x86_64-unknown-linux-musl/release/enman /usr/local/bin/enman
COPY target/x86_64-unknown-linux-musl/release/em /usr/local/bin/em

ENTRYPOINT ["enman"]
```

## 验证发布

在发布前，验证构建的二进制文件是否正常工作：

```bash
# 检查版本
./target/release/enman --version

# 检查帮助
./target/release/enman --help

# 测试主要功能
./target/release/enman list
```

## 版本管理

enman 使用语义化版本控制（Semantic Versioning）：

- 主版本号：重大更改，不向后兼容
- 次版本号：新增功能，向后兼容
- 修订号：错误修复，向后兼容

每次发布时，请确保更新 [CHANGELOG.md](file://f:\enman\CHANGELOG.md) 文件，记录所有重要更改。

## 发布检查清单

- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] 版本号已更新 ([Cargo.toml](file://f:\enman\Cargo.toml))
- [ ] CHANGELOG.md 已更新
- [ ] 二进制文件在目标平台上测试通过
- [ ] 安装和卸载过程测试通过
- [ ] 所有命令别名功能正常
- [ ] 跨平台兼容性验证
- [ ] 发布包大小合理
- [ ] 发布包包含必要的文档