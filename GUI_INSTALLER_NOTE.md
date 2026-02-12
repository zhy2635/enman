# 关于 enman GUI 安装程序

## 当前状态

我们目前正在开发 enman 的图形界面安装程序，但目前还没有准备好用于生产环境。在尝试构建过程中，我们遇到了一些与 Tauri 框架相关的复杂性问题，这些问题需要更多时间来解决。

## 推荐的安装方式

尽管目前没有 GUI 安装程序，但 enman 提供了简单快捷的安装方式：

### 1. 使用 Cargo（推荐）

如果您已经安装了 Rust 工具链，这是最简单的方式：

```bash
cargo install enman
```

### 2. 使用一键安装脚本

#### Windows 用户
```powershell
# 在 PowerShell 中运行
Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/yourname/enman/main/install.ps1')
```

#### macOS/Linux 用户
```bash
# 在终端中运行
curl -fsSL https://raw.githubusercontent.com/yourname/enman/main/install.sh | bash
```

### 3. 从源码构建

如果您想从源码构建：

```bash
git clone https://github.com/yourname/enman.git
cd enman
cargo build --release
```

## 未来计划

我们计划在未来发布以下安装选项：

1. **预构建二进制发行版**：为不同平台提供预构建的二进制文件
2. **GUI 安装程序**：为非技术用户提供的图形界面安装程序
3. **包管理器集成**：与 Homebrew、Chocolatey、APT 等包管理器集成

## 为什么暂时没有 GUI 安装程序？

在开发过程中，我们发现：

1. **Tauri 版本兼容性问题**：我们尝试使用 Tauri 框架构建 GUI 安装程序，但在配置方面遇到了版本兼容性问题
2. **图标格式要求**：Windows 平台需要特定格式的图标文件，这增加了构建的复杂性
3. **资源开销**：维护 GUI 安装程序需要额外的开发和维护资源

当前，我们决定优先完善核心功能和 CLI 体验，稍后再开发 GUI 安装程序。

## 立即开始使用

即使没有 GUI 安装程序，enman 仍然非常容易安装和使用。只需按照 [安装指南](./INSTALL.md) 中的说明操作即可快速开始使用。

## 反馈

如果您对 GUI 安装程序有特别的需求或想法，欢迎在 GitHub 上提出 issue 或 PR。我们重视用户的反馈，并将持续改进 enman 的用户体验。