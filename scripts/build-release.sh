#!/bin/bash

# Shell 脚本：构建发布版本的 enman
echo "Building release version of enman..."

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo (Rust) is not installed or not in PATH."
    echo "Please install Rust from https://www.rust-lang.org/tools/install"
    exit 1
fi

# 构建发布版本
echo "Building release binaries..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# 创建发布目录
release_dir="releases"
mkdir -p $release_dir

# 获取当前版本
version=$(grep '^version =' Cargo.toml | head -n1 | cut -d '"' -f2)
arch=$(uname -m)
platform=$(uname -s | tr '[:upper:]' '[:lower:]')
release_name="enman-v${version}-${platform}-${arch}"

dist_dir="${release_dir}/${release_name}"
rm -rf $dist_dir
mkdir -p $dist_dir

# 复制二进制文件
src_dir="target/release"
cp "${src_dir}/enman" "${dist_dir}/"
cp "${src_dir}/em" "${dist_dir}/"

# 复制文档
cp "README.md" "${dist_dir}/"
cp "LICENSE" "${dist_dir}/" 2>/dev/null || echo "LICENSE file not found, skipping..."

echo "Release built successfully in ${dist_dir}"
echo "Files included:"
ls -la $dist_dir

# 创建 TAR.GZ 存档
archive_path="${release_dir}/${release_name}.tar.gz"
tar -czf $archive_path -C $release_dir $(basename $dist_dir)
echo "Archive created: $archive_path"