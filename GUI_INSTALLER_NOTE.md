# 关于 enman GUI 安装程序

## 当前状态

我们现在已经开发了 enman 的图形界面安装程序！Windows 用户可以从 [Releases](https://github.com/zhy2635/enman/releases) 页面下载 `enman-x86_64-pc-windows-msvc.exe` 文件并运行安装程序。

安装程序使用 Inno Setup 构建，提供多语言支持和自动环境变量配置。

## 安装程序特性

- **多语言支持**：安装过程中可以选择界面语言以及命令行工具的默认语言
- **自动PATH配置**：可选择将 enman 自动添加到系统 PATH 环境变量
- **一键安装**：简单几步即可完成安装，无需手动配置
- **配置持久化**：安装时可选择命令行工具的语言偏好，并保存至配置文件

## 推荐的安装方式

### Windows 用户（推荐）
下载预构建的安装程序：

1. 访问 [GitHub Releases](https://github.com/zhy2635/enman/releases) 页面
2. 下载 `enman-x86_64-pc-windows-msvc.exe`
3. 运行安装程序并按照提示完成安装
4. 重启终端以使环境变量生效

### 使用 Cargo（跨平台推荐）

如果您已经安装了 Rust 工具链：

```bash
cargo install enman
```

### 使用一键安装脚本

#### Windows 用户
```powershell
# 在 PowerShell 中运行
Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/zhy2635/enman/main/install.ps1')
```

#### macOS/Linux 用户
```bash
# 在终端中运行
curl -fsSL https://raw.githubusercontent.com/zhy2635/enman/main/install.sh | bash
```

### 从源码构建

如果您想从源码构建：

```bash
git clone https://github.com/zhy2635/enman.git
cd enman
cargo build --release
```

## 构建安装程序

如果您想自己构建 Windows 安装程序：

1. 安装 [Inno Setup](http://www.jrsoftware.org/isinfo.php)
2. 使用项目根目录的 [setup.iss](file:///f:/enman/setup.iss) 配置文件构建安装程序

```batch
# 使用命令行构建安装程序
ISCC.exe setup.iss
```

安装程序会自动将 enman 添加到系统 PATH，并创建必要的目录结构。

## 主要特性

enman 提供了许多强大的功能：

- **纯净的命令输出**：最新版本提供纯净的工具输出，不再显示任何调试信息，如 `[DEBUG]`、`[SHIM]` 或 `[LOCAL]` 标签
- **透明的命令拦截**：无缝切换工具版本而不影响使用体验
- **项目级配置**：支持 `.enmanrc` 文件进行项目特定的环境配置
- **跨平台支持**：在 Windows、macOS 和 Linux 上提供一致的体验

## 立即开始使用

安装完成后，使用以下命令初始化 enman：

```bash
enman init
```

## 反馈

如果您对 GUI 安装程序有任何反馈或建议，欢迎在 GitHub 上提出 issue 或 PR。我们重视用户的反馈，并将持续改进 enman 的用户体验。