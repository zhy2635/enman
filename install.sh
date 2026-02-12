#!/bin/bash
#
# enman 安装脚本 for Unix/Linux/macOS
#
# 用法:
#   在终端中运行:
#     curl -fsSL https://raw.githubusercontent.com/yourname/enman/main/install.sh | bash
#   或者下载脚本后运行:
#     chmod +x install.sh
#     ./install.sh

set -e

# 默认安装路径
DEFAULT_INSTALL_PATH="$HOME/.enman"
INSTALL_PATH="${ENMAN_INSTALL_PATH:-$DEFAULT_INSTALL_PATH}"
SHIMS_PATH="$INSTALL_PATH/shims"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${GREEN}$1${NC}"
}

print_info() {
    echo -e "${BLUE}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

print_error() {
    echo -e "${RED}$1${NC}" >&2
}

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    print_error "未检测到 Cargo。请先安装 Rust 工具链："
    print_info "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    print_info "然后重启终端以继续安装。"
    exit 1
fi

print_header "正在安装 enman..."

# 克隆或更新 enman 仓库
REPO_PATH="/tmp/enman-repo"
if [ -d "$REPO_PATH" ]; then
    print_info "更新 enman 仓库..."
    cd "$REPO_PATH"
    git pull origin main
else
    print_info "克隆 enman 仓库..."
    git clone "https://github.com/yourname/enman.git" "$REPO_PATH"
    cd "$REPO_PATH"
fi

# 构建 enman
print_info "正在构建 enman..."
cargo build --release

# 创建安装目录
print_info "创建安装目录..."
mkdir -p "$INSTALL_PATH"
BIN_PATH="$INSTALL_PATH/bin"
mkdir -p "$BIN_PATH"

# 复制二进制文件
cp "$REPO_PATH/target/release/enman" "$BIN_PATH/"
cp "$REPO_PATH/target/release/em" "$BIN_PATH/"

# 创建 shims 目录
mkdir -p "$SHIMS_PATH"

# 初始化 enman
print_info "初始化 enman..."
"$BIN_PATH/enman" init

# 检查 shell 类型并添加到 PATH
SHELL_PROFILE=""
if [ -n "$ZSH_VERSION" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        SHELL_PROFILE="$HOME/.zshrc"  # macOS Catalina+ defaults to zsh
    else
        SHELL_PROFILE="$HOME/.bashrc"
    fi
fi

# 如果没有自动检测到，尝试常见配置文件
if [ -z "$SHELL_PROFILE" ]; then
    if [ -f "$HOME/.zshrc" ]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        SHELL_PROFILE="$HOME/.bash_profile"
    fi
fi

# 添加到 PATH（如果需要）
if [[ ":$PATH:" != *":$SHIMS_PATH:"* ]]; then
    print_info "添加到 PATH..."
    
    if [ -n "$SHELL_PROFILE" ] && [ -f "$SHELL_PROFILE" ]; then
        # 检查是否已经添加过
        if ! grep -q "$SHIMS_PATH" "$SHELL_PROFILE"; then
            echo "" >> "$SHELL_PROFILE"
            echo "# enman shims" >> "$SHELL_PROFILE"
            echo "export PATH=\"$SHIMS_PATH:\$PATH\"" >> "$SHELL_PROFILE"
            print_info "已将 enman 添加到 $SHELL_PROFILE"
        else
            print_info "enman 已存在于 $SHELL_PROFILE 中"
        fi
    else
        print_warning "无法自动检测或更新 shell 配置文件。"
        print_info "请手动将以下行添加到您的 shell 配置文件中："
        print_info "export PATH=\"$SHIMS_PATH:\$PATH\""
    fi
    
    # 为当前会话也添加
    export PATH="$SHIMS_PATH:$PATH"
else
    print_info "PATH 已包含 enman shims 路径。"
fi

print_header "enman 安装完成！"
print_info "请运行 'source $SHELL_PROFILE' 或重启终端。"
echo ""
print_info "现在您可以使用以下命令开始:"
print_info "  enman install node@18.17.0  # 安装 Node.js"
print_info "  enman global node@18.17.0   # 设置全局版本"
print_info "  node --version              # 验证安装"