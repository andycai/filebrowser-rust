#!/bin/bash

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}   Rust 文件浏览器编译脚本${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 创建输出目录
mkdir -p build

# 编译函数
build() {
    local target=$1
    local output=$2

    echo -e "${YELLOW}编译 ${target}...${NC}"

    cargo build --release --target "${target}" --bin filebrowser

    if [ $? -eq 0 ]; then
        # 复制到 build 目录
        if [ -f "target/${target}/release/filebrowser" ]; then
            cp "target/${target}/release/filebrowser" "build/${output}"
        elif [ -f "target/${target}/release/filebrowser.exe" ]; then
            cp "target/${target}/release/filebrowser.exe" "build/${output}"
        fi

        size=$(ls -lh "build/${output}" | awk '{print $5}')
        echo -e "${GREEN}✓ ${target} 编译成功 (${size})${NC}"
    else
        echo -e "${RED}✗ ${target} 编译失败${NC}"
        return 1
    fi
}

# 编译当前平台
build_current() {
    local platform=$1
    local output=$2

    echo -e "${YELLOW}编译 ${platform}（当前平台）...${NC}"
    cargo build --release --bin filebrowser

    if [ $? -eq 0 ]; then
        cp target/release/filebrowser "build/${output}"
        size=$(ls -lh "build/${output}" | awk '{print $5}')
        echo -e "${GREEN}✓ ${platform} 编译成功 (${size})${NC}"
    else
        echo -e "${RED}✗ ${platform} 编译失败${NC}"
        return 1
    fi
}

# 检测当前平台
if [[ "$OSTYPE" == darwin* ]]; then
    CURRENT_OS="macOS"
    if [[ "$(uname -m)" == "arm64" ]]; then
        CURRENT_ARCH="ARM64"
        CURRENT_TARGET="filebrowser-darwin-arm64"
    else
        CURRENT_ARCH="Intel"
        CURRENT_TARGET="filebrowser-darwin-amd64"
    fi
elif [[ "$OSTYPE" == linux* ]]; then
    CURRENT_OS="Linux"
    CURRENT_TARGET="filebrowser-linux-amd64"
else
    CURRENT_OS="Unknown"
    CURRENT_TARGET="filebrowser"
fi

# 编译目标
echo "开始编译..."
echo ""

# 1. 编译当前平台（最重要）
build_current "${CURRENT_OS} (${CURRENT_ARCH})" "${CURRENT_TARGET}"
echo ""

# 2. macOS 交叉编译
if [[ "$OSTYPE" == darwin* ]]; then
    # 如果在 macOS 上，编译另一个架构
    if [[ "$(uname -m)" == "x86_64" ]]; then
        # Intel 上编译 ARM64
        echo -e "${YELLOW}编译 macOS ARM64（交叉编译）...${NC}"
        if cargo build --release --target aarch64-apple-darwin --bin filebrowser 2>/dev/null; then
            cp target/aarch64-apple-darwin/release/filebrowser build/filebrowser-darwin-arm64
            size=$(ls -lh build/filebrowser-darwin-arm64 | awk '{print $5}')
            echo -e "${GREEN}✓ aarch64-apple-darwin 编译成功 (${size})${NC}"
        else
            echo -e "${RED}✗ aarch64-apple-darwin 编译失败（可能需要安装交叉编译工具）${NC}"
        fi
    else
        # ARM64 上编译 Intel
        echo -e "${YELLOW}编译 macOS Intel（交叉编译）...${NC}"
        if cargo build --release --target x86_64-apple-darwin --bin filebrowser 2>/dev/null; then
            cp target/x86_64-apple-darwin/release/filebrowser build/filebrowser-darwin-amd64
            size=$(ls -lh build/filebrowser-darwin-amd64 | awk '{print $5}')
            echo -e "${GREEN}✓ x86_64-apple-darwin 编译成功 (${size})${NC}"
        else
            echo -e "${RED}✗ x86_64-apple-darwin 编译失败（可能需要安装交叉编译工具）${NC}"
        fi
    fi
    echo ""
fi

# 3. Windows 交叉编译
echo -e "${YELLOW}编译 Windows（交叉编译）...${NC}"
echo -e "${YELLOW}提示: 如果失败，请先运行: rustup target add x86_64-pc-windows-gnu${NC}"

if cargo build --release --target x86_64-pc-windows-gnu --bin filebrowser 2>/dev/null; then
    cp target/x86_64-pc-windows-gnu/release/filebrowser.exe build/filebrowser-windows-amd64.exe
    size=$(ls -lh build/filebrowser-windows-amd64.exe | awk '{print $5}')
    echo -e "${GREEN}✓ x86_64-pc-windows-gnu 编译成功 (${size})${NC}"
else
    echo -e "${RED}✗ x86_64-pc-windows-gnu 编译失败${NC}"
    echo -e "${YELLOW}要交叉编译 Windows，请执行以下步骤:${NC}"
    echo -e "${YELLOW}  1. rustup target add x86_64-pc-windows-gnu${NC}"
    echo -e "${YELLOW}  2. 在 macOS 上: brew install mingw-w64${NC}"
    echo -e "${YELLOW}  3. 在 Linux 上: sudo apt-get install mingw-w64${NC}"
fi
echo ""

# 4. Linux 交叉编译
echo -e "${YELLOW}编译 Linux（交叉编译）...${NC}"
if cargo build --release --target x86_64-unknown-linux-gnu --bin filebrowser 2>/dev/null; then
    cp target/x86_64-unknown-linux-gnu/release/filebrowser build/filebrowser-linux-amd64
    size=$(ls -lh build/filebrowser-linux-amd64 | awk '{print $5}')
    echo -e "${GREEN}✓ x86_64-unknown-linux-gnu 编译成功 (${size})${NC}"
else
    echo -e "${RED}✗ x86_64-unknown-linux-gnu 编译失败（跳过）${NC}"
fi
echo ""

# 创建发布包
echo -e "${YELLOW}创建发布包...${NC}"
cd build

# 只打包可执行文件，不包括已存在的压缩文件
for file in filebrowser-darwin-amd64 filebrowser-darwin-arm64 filebrowser-linux-amd64 filebrowser-linux-arm64 filebrowser-windows-amd64.exe; do
    if [ -f "$file" ]; then
        if [[ "$file" =~ darwin ]] || [[ "$file" =~ linux ]]; then
            tar -czf "${file}.tar.gz" "$file" 2>/dev/null && echo "  ✓ ${file}.tar.gz"
        elif [[ "$file" =~ windows ]]; then
            zip "${file%.exe}.zip" "$file" 2>/dev/null && echo "  ✓ ${file%.exe}.zip"
        fi
    fi
done

cd ..

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}   编译完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "输出目录: build/"
echo ""
ls -lh build/ 2>/dev/null | grep -v "tar.gz\|zip" || echo "  (空)"
echo ""
echo -e "${YELLOW}发布包:${NC}"
ls -lh build/*.tar.gz build/*.zip 2>/dev/null || echo "  (无)"
