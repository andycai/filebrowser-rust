# 快速开始指南 - Rust FileBrowser

## 前提条件

- Rust 1.70 或更高版本
- 终端访问权限

## 检查 Rust 环境

```bash
rustc --version
cargo --version
```

如果未安装 Rust，请访问 https://rustup.rs/ 安装。

## 三种启动方式

### 方式 1：开发模式（最简单）

```bash
# 进入项目目录
cd filebrowser-rust

# 直接运行
cargo run
```

### 方式 2：编译后运行（推荐）

```bash
# 编译发布版本
cargo build --release

# 运行
./target/release/filebrowser
```

### 方式 3：使用 Makefile

```bash
# 编译并运行
make build-release
make start
```

## 配置

编辑 `config.json`：

```json
{
  "rootDir": ".",
  "port": 8080,
  "staticDir": "./static"
}
```

## 访问

打开浏览器：`http://localhost:8080`

## 性能特点

- **小文件（<10MB）**：一次性加载
- **大文件（≥10MB）**：自动分页，每页 1000 行
- **内存占用**：约 5-10 MB（比 Go 版本低 50%）
- **二进制大小**：约 3 MB（比 Go 版本小 60%）

## 常用命令

```bash
# 开发模式运行
make run

# 实时监控代码变化
make dev

# 检查代码
make check

# 格式化代码
make fmt

# 查看日志
make logs

# 停止服务
make stop
```

## 对比 Go 版本

| 特性 | Go | Rust |
|------|-----|------|
| 二进制大小 | 8 MB | 3 MB |
| 内存占用 | 15-20 MB | 5-10 MB |
| 启动时间 | 1-2 ms | <1 ms |
| 内存安全 | GC | 编译时保证 |

更多信息请参考 `README.md`
