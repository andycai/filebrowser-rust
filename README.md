# Rust FileBrowser

一个使用 Rust 语言开发的高性能 Web 文件浏览器，支持浏览目录、查看文本文件内容，并对大文件进行了性能优化。

## 特性

### 1. 核心功能
- **多根目录支持**: 支持配置多个根目录，可在不同目录间快速切换
- 浏览文件和目录
- 查看文本文件内容
- 大文件分页加载（≥10MB）
- 全文搜索功能
- 直接 URL 访问文件
- 面包屑导航

### 2. 性能优化
- 流式文件读取
- 大文件分页处理（每页 1000 行）
- 异步 I/O 处理
- 零成本抽象

### 3. 技术栈
- **HTTP 框架**: Axum 0.7 + Tokio
- **序列化**: Serde
- **路径处理**: path-clean, pathdiff
- **日志**: tracing
- **跨平台**: 支持 macOS、Linux、Windows

## 编译和部署

### 方式一：直接运行（开发模式）

```bash
# 运行
cargo run

# 或先编译再运行
cargo build --release
./target/release/filebrowser
```

### 方式二：使用 Makefile

```bash
# 编译（开发版本）
make build

# 编译（发布版本）
make build-release

# 运行（开发模式）
make run

# 实时监控文件变化（开发模式）
make dev
```

### 方式三：交叉编译（多平台）

```bash
# 编译所有平台
./build.sh

# 或使用 make
make build-all
```

编译完成后，`build/` 目录将包含以下文件：

| 平台 | 架构 | 文件名 |
|------|------|--------|
| macOS | Intel (amd64) | `filebrowser-darwin-amd64` |
| macOS | Apple Silicon (arm64) | `filebrowser-darwin-arm64` |
| Linux | AMD64 | `filebrowser-linux-amd64` |
| Linux | ARM64 | `filebrowser-linux-arm64` |
| Windows | AMD64 | `filebrowser-windows-amd64.exe` |

### 方式四：使用服务管理脚本

#### Linux / macOS

```bash
# 启动服务
make start

# 停止服务
make stop

# 重启服务
make restart

# 查看状态
make status

# 查看日志（实时）
make logs
```

#### Windows

```cmd
REM 启动服务
service.bat start

REM 停止服务
service.bat stop

REM 重启服务
service.bat restart

REM 查看状态
service.bat status

REM 查看日志
service.bat logs
```

### 方式五：安装为系统服务（Linux systemd）

在 Linux 系统上，可以安装为 systemd 系统服务：

```bash
# 1. 先编译
make build-all

# 2. 安装系统服务（需要 sudo）
sudo ./install.sh
```

安装后可以使用 systemctl 管理：

```bash
# 启动服务
sudo systemctl start filebrowser

# 停止服务
sudo systemctl stop filebrowser

# 重启服务
sudo systemctl restart filebrowser

# 查看状态
sudo systemctl status filebrowser

# 设置开机自启
sudo systemctl enable filebrowser

# 查看日志
sudo journalctl -u filebrowser -f
```

## 配置

编辑 `config.json` 文件：

```json
{
  "rootDirs": [
    {
      "name": "项目目录",
      "path": "."
    },
    {
      "name": "用户主目录",
      "path": "/Users/username"
    },
    {
      "name": "系统根目录",
      "path": "/"
    }
  ],
  "port": 8080,
  "staticDir": "./static"
}
```

**配置说明**:
- `rootDirs`: 根目录配置数组（支持多个根目录）
  - `name`: 显示名称（在界面上显示的名称）
  - `path`: 实际文件系统路径（支持相对路径和绝对路径）
- `port`: 服务器监听端口
- `staticDir`: 静态文件目录路径

**根目录切换**:
- 界面顶部有根目录选择下拉框
- 切换根目录后自动跳转到新根目录的首页
- 每个根目录的浏览历史独立维护

**向后兼容**:
如果仍使用旧的单根目录配置格式，系统会自动兼容：

```json
{
  "rootDir": "/home/user/documents",
  "port": 8080,
  "staticDir": "./static"
}
```

## 访问

打开浏览器访问: `http://localhost:8080`

### 直接访问文件

可以通过 URL 直接访问文件：

```
http://localhost:8080/view/path/to/file.txt
```

### 切换根目录

使用页面顶部的根目录选择器下拉框：
- 每个根目录的浏览状态独立维护
- 切换根目录会自动返回到该根目录的首页
- 支持在不同根目录间快速切换

## 性能对比

### Rust vs Go

| 特性 | Go 版本 | Rust 版本 |
|------|---------|-----------|
| 二进制大小 | ~8 MB | ~3 MB (stripped) |
| 内存占用 | ~15-20 MB | ~5-10 MB |
| 启动时间 | ~1-2 ms | <1 ms |
| 性能 | 高 | 极高 |
| 安全性 | 安全 | 内存安全保证 |
| 功能对等性 | 完整功能 | 完整功能（包括多根目录） |

### 优化特点

1. **零拷贝**: 使用 Rust 的所有权系统，避免不必要的数据复制
2. **零成本抽象**: 高级特性在编译时优化为高效的机器码
3. **内存安全**: 编译时保证内存安全，无运行时开销
4. **异步 I/O**: 基于 Tokio 的高性能异步运行时

## API 接口

### 1. 获取根目录列表

**请求**: `GET /api/roots`

**响应**:
```json
[
  {
    "name": "项目目录",
    "path": "/absolute/path/to/project"
  },
  {
    "name": "用户主目录",
    "path": "/Users/username"
  }
]
```

### 2. 获取目录列表

**请求**: `GET /api/list?path=<path>&root=<rootIndex>`

**参数**:
- `path`: 目录路径（相对于根目录）
- `root`: 根目录索引（可选，默认为 0）

**响应**:
```json
[
  {
    "name": "文件名",
    "path": "相对路径",
    "isDir": false,
    "size": 1024,
    "modTime": "2024-01-01T12:00:00Z",
    "extension": "txt"
  }
]
```

### 3. 查看文件内容

**请求**: `GET /api/view?path=<path>&page=<page>&root=<rootIndex>`

**参数**:
- `path`: 文件路径（相对于根目录）
- `page`: 页码（可选，默认为 1）
- `root`: 根目录索引（可选，默认为 0）

**响应**:
```json
{
  "name": "文件名",
  "path": "相对路径",
  "size": 1024000,
  "totalLines": 150000,
  "lines": ["第1行", "第2行", "..."],
  "page": 1,
  "totalPages": 150,
  "isPartial": true
}
```

### 4. 搜索文件内容

**请求**: `GET /api/search?path=<path>&q=<query>&root=<rootIndex>`

**参数**:
- `path`: 文件路径（相对于根目录）
- `q`: 搜索关键词
- `root`: 根目录索引（可选，默认为 0）

**响应**:
```json
[
  {
    "lineNumber": 42,
    "page": 1,
    "line": "包含搜索关键词的行内容"
  }
]
```

## 项目结构

```
filebrowser-rust/
├── Cargo.toml              # Rust 项目配置
├── Makefile                # 构建工具
├── config.json             # 配置文件
├── build.sh                # 交叉编译脚本
├── service.sh              # Linux/macOS 服务管理脚本
├── service.bat             # Windows 服务管理脚本
├── install.sh              # Linux systemd 安装脚本
├── build/                  # 编译输出目录
├── src/
│   ├── main.rs             # 主程序和 HTTP 服务器
│   ├── config.rs           # 配置文件加载
│   └── scanner.rs          # 文件扫描器
└── static/                 # 静态文件目录
    ├── index.html          # 前端页面
    ├── style.css           # 样式文件
    └── app.js              # 前端 JavaScript 逻辑
```

## 开发

### 环境要求

- Rust 1.70 或更高版本
- Cargo（随 Rust 一起安装）

### 开发命令

```bash
# 检查代码
make check

# 格式化代码
make fmt

# 运行测试
make test

# 实时监控文件变化
make dev

# 更新依赖
make update
```

### 添加依赖

```bash
cargo add <crate_name>
```

### 构建优化

```bash
# 发布版本编译（已启用 LTO 和代码剥离）
cargo build --release

# 使用 UPX 进一步压缩（需要先安装 UPX）
upx --best --lzma target/release/filebrowser
```

## 安全性

- 防止目录遍历攻击（`..`）
- 限制在配置的根目录内
- 所有路径都经过验证和清理
- 使用 Rust 的类型系统保证内存安全

## 故障排查

**编译失败**

```bash
# 检查 Rust 版本
rustc --version

# 更新 Rust
rustup update

# 清理并重新编译
make clean
make build
```

**服务启动失败**

```bash
# 查看日志
make logs

# 或直接查看日志文件
cat filebrowser.log
```

**端口被占用**

```bash
# 修改 config.json 中的 port
# 或查找占用进程
lsof -i :8080
```

## 常见问题

### Q: 如何修改每页显示的行数？
A: 修改 `src/scanner.rs` 中的 `LINES_PER_PAGE` 常量。

### Q: 如何修改大文件的阈值？
A: 修改 `src/scanner.rs` 中的 `LARGE_FILE_THRESHOLD` 常量。

### Q: 支持哪些文件类型？
A: 支持所有文本文件。二进制文件会显示乱码，不建议查看。

### Q: 可以同时查看多个文件吗？
A: 当前版本只支持查看单个文件，多文件标签页功能可以在未来版本中添加。

### Q: 如何配置多个根目录？
A: 在 `config.json` 中的 `rootDirs` 数组中添加多个根目录配置，每个目录需要指定 `name`（显示名称）和 `path`（实际路径）。

### Q: 切换根目录会影响浏览历史吗？
A: 不会。每个根目录的浏览历史独立维护，切换根目录后，之前根目录的浏览状态会被保留。

### Q: 可以在不同的根目录之间共享文件吗？
A: 不可以。每个根目录是独立的，只能访问该根目录及其子目录下的文件。

### Q: Rust 版本与 Go 版本功能是否完全一致？
A: 是的，Rust 版本与 Go 版本功能完全对等，包括多根目录支持、大文件分页、全文搜索等所有核心功能。

## 为什么选择 Rust？

与 Go 版本相比，Rust 版本具有以下优势：

1. **更小的二进制大小**: 编译后只有约 3 MB（Go 版本约 8 MB）
2. **更低的内存占用**: 运行时内存占用减少 50%
3. **更高的性能**: 零成本抽象和更好的优化
4. **内存安全**: 编译时保证，无运行时垃圾回收
5. **并发安全**: 编译时检查数据竞争

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
