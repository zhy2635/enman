# enman - Environment Manager

一个统一的开发环境管理工具，用于轻松管理多个开发工具的版本，如 Node.js、Python、Java、MySQL、MariaDB 等。

[![Crates.io](https://img.shields.io/crates/v/enman.svg)](https://crates.io/crates/enman)
[![License](https://img.shields.io/crates/l/enman.svg)](https://github.com/zhy2635/enman/blob/main/LICENSE)

## 特性

- ✅ 轻松安装和切换多个工具的不同版本
- ✅ 透明的命令拦截和自动版本切换
- ✅ 全局和项目级环境配置
- ✅ 项目级配置文件支持（`.enmanrc`）
- ✅ 一致的跨平台体验（Windows、macOS、Linux）
- ✅ 简洁直观的命令行界面
- ✅ 无调试输出干扰的纯净命令输出
- ✅ Windows 图形安装程序（带有多语言支持）
- ✅ 单命令安装脚本（见下方）

## 安装

### 使用 Cargo 安装

```bash
cargo install enman
```

### 使用一键安装脚本

#### Windows
```powershell
# 在 PowerShell 中运行
Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/zhy2635/enman/main/install.ps1')
```

#### macOS/Linux
```bash
# 在终端中运行
curl -fsSL https://raw.githubusercontent.com/zhy2635/enman/main/install.sh | bash
```

### 从源码构建

```bash
git clone https://github.com/zhy2635/enman.git
cd enman
cargo install --path .
```

### 下载预构建二进制文件

访问 [Releases](https://github.com/zhy2635/enman/releases) 页面下载适合你系统的预构建二进制文件。

**Windows 用户**: 下载 `enman-x86_64-pc-windows-msvc.exe` 文件并运行安装程序。安装程序提供多语言支持和自动环境变量配置。

## GUI 安装程序

我们为 Windows 用户提供了图形界面安装程序，具有以下特性：

- **多语言支持**：安装过程中可以选择界面语言以及命令行工具的默认语言
- **自动PATH配置**：可选择将 enman 自动添加到系统 PATH 环境变量
- **一键安装**：简单几步即可完成安装，无需手动配置
- **配置持久化**：安装时可选择命令行工具的语言偏好，并保存至配置文件

请参阅 [GUI 安装程序说明](./GUI_INSTALLER_NOTE.md) 了解更多信息。

## 快速开始

1. 初始化 enman：

```bash
enman init
# 或使用别名
em init
```

2. 安装工具：

```bash
# 安装特定版本的工具
enman install node@18.17.0
enman install python@3.11.5
enman install java@17

# 列出可用版本
enman list node --remote
enman list python --remote
enman list java --remote
```

3. 设置全局版本：

```bash
# 设置全局默认版本
enman global node@18.17.0
enman global python@3.11.5
enman global java@17
```

4. 在项目中使用特定版本：

```bash
# 创建项目配置文件
echo '[tools]' > .enmanrc
echo 'node = "16.14.0"' >> .enmanrc
echo 'python = "3.10.9"' >> .enmanrc

# 应用配置
enman config apply
```

## 命令详解

### `install` 或 `in`

安装指定版本的工具：

```bash
enman install node@18.17.0
enman install python@3.11.5
enman install java@17
```

### `use`

临时切换当前会话中的工具版本：

```bash
enman use node@16.14.0
node --version  # 输出: v16.14.0 (无额外调试信息)
```

### `list` 或 `ls`

列出已安装或可用的工具版本：

```bash
enman list node        # 列出已安装的 Node.js 版本
enman list node --remote  # 列出可安装的 Node.js 版本
enman list --available   # 列出所有可用的工具
```

### `global`

设置全局默认工具版本：

```bash
enman global node@18.17.0
enman global python@3.11.5
```

### `config`

管理项目级配置：

```bash
enman config init      # 创建默认 .enmanrc 文件
enman config show      # 显示当前配置
enman config apply     # 应用当前目录的 .enmanrc 配置
```

### `init`

初始化 enman 环境：

```bash
enman init
```

这将在您的主目录中创建必要的目录结构，并生成 shell 集成脚本。

## 项目级配置

enman 支持项目级配置文件 `.enmanrc`，格式如下：

```toml
[tools]
node = "18.17.0"
python = "3.11.5"
java = "17"
```

当进入项目目录时，enman 会自动应用这些配置。

## Shim 机制和纯净输出

enman 使用 shim 机制来拦截命令并根据全局或项目配置自动切换工具版本。所有受支持的工具命令都会通过 `~/.enman/shims` 目录中的 shim 可执行文件进行路由。

**重要更新**: 最新版本的 enman 现在提供纯净的工具输出，不再显示任何调试信息，如 `[DEBUG]`、`[SHIM]` 或 `[LOCAL]` 标签。这让工具输出保持干净和可预测。

## 高级用法

### 使用别名

enman 提供了许多方便的别名：

- `em` 是 `enman` 的别名
- `in` 是 `install` 的别名
- `ls` 是 `list` 的别名
- `un` 是 `uninstall` 的别名

### 支持的工具

目前支持的工具包括：
- Node.js
- Python
- Java
- MySQL
- MariaDB
- 更多工具即将推出...

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 许可证

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## 支持

如果遇到问题，请查看 [GitHub Issues](https://github.com/zhy2635/enman/issues) 或提交新问题。

---

更多信息请参阅：
- [安装指南](./INSTALL.md)
- [配置说明](./CONFIGURATION.md)
- [发布和打包流程](./PUBLISHING.md)
- [GUI 安装程序说明](./GUI_INSTALLER_NOTE.md)