# enman 安装指南

enman 是一个统一的开发环境管理工具，用于管理各种开发工具的版本。本文档提供了在不同平台上安装 enman 的详细说明。

## 系统要求

- Windows 7 SP1+, macOS 10.12+, 或 Linux 发行版
- Rust 1.65+ (如果从源码构建)
- Git

## 安装方式

### 方式一：使用 Cargo 安装（推荐）

如果你已经安装了 Rust 工具链，这是最简单的方法：

```bash
# 从 crates.io 安装
cargo install enman

# 或者从 GitHub 安装最新版本
cargo install --git https://github.com/zhy2635/enman.git
```

### 方式二：下载预构建的二进制文件

1. 访问 [GitHub Releases](https://github.com/zhy2635/enman/releases) 页面
2. 根据你的操作系统下载对应的预构建二进制文件：
   - Windows: `enman-x86_64-pc-windows-msvc.exe` - 下载并运行安装程序
   - macOS: `enman-x86_64-apple-darwin.tar.gz`
   - Linux: `enman-x86_64-unknown-linux-gnu.tar.gz`
3. 对于 Windows 用户，下载 `.exe` 文件并运行安装程序。安装完成后，重启终端以使环境变量生效。
4. 对于 macOS/Linux，解压并将其添加到系统的 PATH 环境变量中

### 方式三：从源码构建

```bash
# 克隆仓库
git clone https://github.com/zhy2635/enman.git
cd enman

# 构建项目
cargo build --release

# 二进制文件位于 target/release 目录中
# 将其复制到系统 PATH 中的目录
```

### 方式四：Windows 一键安装脚本

如果你使用 Windows，也可以使用 PowerShell 一键安装脚本：

```powershell
# 在 PowerShell 中以管理员身份运行
Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/zhy2635/enman/main/install.ps1')
```

## 首次使用设置

安装完成后，运行以下命令进行初始化：

```bash
# 初始化 enman
enman init

# 或使用别名
em init
```

此命令将创建必要的目录结构并生成 shell 集成脚本。

## 环境配置

为了使 enman 正常工作，你需要确保 `~/.enman/shims` 目录在你的 PATH 环境变量中。

### Windows

使用 PowerShell 添加到 PATH：

```powershell
[Environment]::SetEnvironmentVariable("PATH", "$env:USERPROFILE\.enman\shims;$([Environment]::GetEnvironmentVariable("PATH", "User"))", "User")
```

### macOS/Linux

将以下行添加到你的 shell 配置文件（如 `.bashrc`, `.zshrc`）：

```bash
export PATH="$HOME/.enman/shims:$PATH"
```

然后重新加载配置：

```bash
source ~/.bashrc  # 或 source ~/.zshrc
```

## 验证安装

安装完成后，验证 enman 是否正常工作：

```bash
# 检查版本
enman --version
em --version  # em 是 enman 的别名

# 查看帮助
enman --help
```

## 基本使用

安装并使用第一个工具：

```bash
# 安装 Node.js 18.17.0 版本
enman install node@18.17.0

# 设置为全局默认版本
enman global node@18.17.0

# 验证安装 - 输出将是纯净的版本号，无额外调试信息
node --version
```

## Shim 机制和纯净输出

enman 使用 shim 机制来拦截命令并根据全局或项目配置自动切换工具版本。所有受支持的工具命令都会通过 `~/.enman/shims` 目录中的 shim 可执行文件进行路由。

**重要更新**: 最新版本的 enman 现在提供纯净的工具输出，不再显示任何调试信息，如 `[DEBUG]`、`[SHIM]` 或 `[LOCAL]` 标签。这让工具输出保持干净和可预测。

## 故障排除

### 命令不可用

确保 `~/.enman/shims` 已添加到 PATH 中。

### 权限错误

在 Unix 系统上确保你对 `~/.enman` 目录有适当的读写权限。

### Windows 上的符号链接权限

在某些 Windows 系统上，创建符号链接可能需要管理员权限或启用了"开发者模式"。

## 后续步骤

- 了解如何使用 [enman 配置](./CONFIGURATION.md)
- 了解 [发布和打包流程](./PUBLISHING.md)
- 了解 [GUI 安装程序说明](./GUI_INSTALLER_NOTE.md)

## 支持

如果遇到问题，请查看 [GitHub Issues](https://github.com/zhy2635/enman/issues) 或提交新问题。