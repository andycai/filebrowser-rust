.PHONY: all build run test clean start stop restart status logs help

# 默认目标
all: build

# 编译当前平台（开发版本）
build:
	@echo "编译当前平台（开发版本）..."
	cargo build

# 编译当前平台（发布版本）
build-release:
	@echo "编译当前平台（发布版本）..."
	cargo build --release
	@cp target/release/filebrowser .

# 运行（开发模式）
run:
	@echo "启动文件浏览器（开发模式）..."
	cargo run

# 交叉编译所有平台
build-all:
	@echo "交叉编译所有平台..."
	./build.sh

# 测试
test:
	@echo "运行测试..."
	cargo test

# 清理编译文件
clean:
	@echo "清理编译文件..."
	cargo clean
	rm -rf build/
	rm -f filebrowser filebrowser.exe
	rm -f filebrowser.log filebrowser.pid

# 启动服务
start: build-release
	@echo "启动服务..."
	./service.sh start

# 停止服务
stop:
	@echo "停止服务..."
	./service.sh stop

# 重启服务
restart: build-release
	@echo "重启服务..."
	./service.sh restart

# 查看状态
status:
	@echo "查看服务状态..."
	./service.sh status

# 查看日志
logs:
	@echo "查看日志..."
	./service.sh logs

# 检查代码
check:
	@echo "检查代码..."
	cargo check
	cargo clippy

# 格式化代码
fmt:
	@echo "格式化代码..."
	cargo fmt

# 更新依赖
update:
	@echo "更新依赖..."
	cargo update

# 安装开发依赖
dev-deps:
	@echo "安装开发依赖..."
	cargo install cargo-watch

# 实时重新加载运行（开发模式）
dev:
	@echo "实时监控文件变化并重新编译..."
	cargo watch -x run

# 优化编译（更小更快的二进制文件）
build-optimized:
	@echo "优化编译..."
	cargo build --release
	@upx target/release/filebrowser || echo "UPX not installed, skipping compression"
	@cp target/release/filebrowser .

# 帮助信息
help:
	@echo "可用的 make 命令:"
	@echo ""
	@echo "编译相关:"
	@echo "  make build          - 编译当前平台（开发版本）"
	@echo "  make build-release  - 编译当前平台（发布版本）"
	@echo "  make build-all      - 交叉编译所有平台"
	@echo "  make build-optimized - 优化编译（使用 UPX 压缩）"
	@echo "  make clean          - 清理编译文件"
	@echo ""
	@echo "运行相关:"
	@echo "  make run            - 直接运行（开发模式）"
	@echo "  make dev            - 实时监控并重新编译（开发模式）"
	@echo "  make start          - 启动服务"
	@echo "  make stop           - 停止服务"
	@echo "  make restart        - 重启服务"
	@echo "  make status         - 查看状态"
	@echo "  make logs           - 查看日志"
	@echo ""
	@echo "开发相关:"
	@echo "  make test           - 运行测试"
	@echo "  make check          - 代码检查"
	@echo "  make fmt            - 格式化代码"
	@echo "  make update         - 更新依赖"
	@echo "  make dev-deps       - 安装开发依赖"
	@echo ""
	@echo "其他:"
	@echo "  make help           - 显示此帮助信息"
