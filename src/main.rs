mod config;
mod scanner;

use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use config::Config;
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io,
    path::{Component, Path},
    sync::Arc,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber;

/// 每页显示的行数
const LINES_PER_PAGE: usize = 1000;

/// 大文件阈值（10MB）
const LARGE_FILE_THRESHOLD: u64 = 10 * 1024 * 1024;

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    path: String,
    #[serde(rename = "isDir")]
    is_dir: bool,
    size: u64,
    #[serde(rename = "modTime")]
    mod_time: String,
    extension: Option<String>,
}

/// 文件查看响应
#[derive(Debug, Serialize)]
struct FileViewResponse {
    name: String,
    path: String,
    size: u64,
    #[serde(rename = "totalLines")]
    total_lines: u64,
    lines: Vec<String>,
    page: u32,
    #[serde(rename = "totalPages")]
    total_pages: u32,
    #[serde(rename = "isPartial")]
    is_partial: bool,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize)]
struct SearchResult {
    #[serde(rename = "lineNumber")]
    line_number: u64,
    page: u32,
    line: String,
}

/// 保存文件请求
#[derive(Debug, Deserialize)]
struct SaveRequest {
    path: String,
    content: String,
}

/// 创建请求
#[derive(Debug, Deserialize)]
struct CreateRequest {
    path: String,
    name: String,
}

/// 成功响应
#[derive(Debug, Serialize)]
struct SuccessResponse {
    success: bool,
    message: String,
}

/// 搜索查询参数
#[derive(Debug, Deserialize)]
struct SearchQuery {
    path: String,
    q: String,
}

/// 文件查看查询参数
#[derive(Debug, Deserialize)]
struct FileQuery {
    path: String,
    #[serde(default = "default_page")]
    page: u32,
}

fn default_page() -> u32 {
    1
}

/// 列表查询参数
#[derive(Debug, Deserialize)]
struct ListQuery {
    path: String,
}

/// 根目录查询参数
#[derive(Debug, Deserialize)]
struct RootQuery {
    #[serde(default = "default_root_index")]
    root: usize,
}

fn default_root_index() -> usize {
    0
}

/// 应用状态
#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

/// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // 加载配置
    let config = Config::load("config.json").unwrap_or_else(|e| {
        eprintln!("警告: 无法加载 config.json: {}, 使用默认配置", e);
        Config::default()
    });

    let static_path = config.get_static_path()?;

    info!("文件浏览器启动中...");
    info!("根目录数量: {}", config.root_dirs.len());
    for (i, root_dir) in config.root_dirs.iter().enumerate() {
        let abs_path = fs::canonicalize(&root_dir.path).unwrap_or_else(|_| {
            eprintln!("警告: 无法解析根目录路径: {}", root_dir.path);
            std::path::PathBuf::from(&root_dir.path)
        });
        info!("  [{}] {} -> {}", i, root_dir.name, abs_path.display());
    }
    info!("静态文件目录: {}", static_path.display());
    info!("端口: {}", config.port);

    let port = config.port;

    let state = AppState {
        config: Arc::new(config),
    };

    // 构建路由
    let app = Router::new()
        // API 路由
        .route("/api/list", get(handle_list))
        .route("/api/search", get(handle_search))
        .route("/api/view", get(handle_view))
        .route("/api/roots", get(handle_roots))
        .route("/api/save", axum::routing::post(handle_save))
        .route("/api/delete", axum::routing::get(handle_delete))
        .route("/api/create", axum::routing::post(handle_create))
        .route("/api/createDir", axum::routing::post(handle_create_dir))
        .route("/view/*path", get(handle_view_redirect))
        // 静态文件服务
        .nest_service("/static", ServeDir::new(static_path))
        // 首页
        .route("/", get(handle_index))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动服务器
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("服务器已启动: http://localhost:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}

/// 处理首页
async fn handle_index() -> Response {
    let html = include_str!("../static/index.html");
    Html(html).into_response()
}

/// HTML 响应包装器
struct Html<T>(T);

impl<T> IntoResponse for Html<T>
where
    T: Into<axum::body::Body>,
{
    fn into_response(self) -> Response {
        let mut res = Response::new(self.0.into());
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        res
    }
}

/// 从 URL 参数获取根目录索引
fn get_root_index_from_query(params: &RootQuery) -> usize {
    params.root
}

/// 获取根目录路径（返回绝对路径）
fn get_root_path(state: &AppState, root_index: usize) -> std::path::PathBuf {
    if root_index < state.config.root_dirs.len() {
        let path = &state.config.root_dirs[root_index].path;
        // 尝试转换为绝对路径
        fs::canonicalize(path).unwrap_or_else(|_| std::path::PathBuf::from(path))
    } else {
        let path = &state.config.root_dirs[0].path;
        fs::canonicalize(path).unwrap_or_else(|_| std::path::PathBuf::from(path))
    }
}

/// 处理 /view/ 路径的重定向
async fn handle_view_redirect(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    Query(params): Query<RootQuery>,
) -> Response {
    let root_index = get_root_index_from_query(&params);
    let root_path = get_root_path(&state, root_index);

    // 解码路径
    let decoded_path = percent_encoding::percent_decode_str(&path)
        .decode_utf8()
        .unwrap_or_default();

    // 验证路径
    match validate_and_resolve_path(&root_path, decoded_path.as_ref()) {
        Ok(full_path) => {
            if full_path.is_file() {
                // 返回带有 JavaScript 重定向的 HTML
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>文件浏览器</title>
</head>
<body>
    <script>
        window.location.href = '/?file={}';
    </script>
</body>
</html>"#,
                    urlencoding::encode(&path)
                );
                Html(html).into_response()
            } else {
                (StatusCode::NOT_FOUND, "不是文件").into_response()
            }
        }
        Err(e) => {
            error!("路径验证失败: {}", e);
            (StatusCode::NOT_FOUND, "文件不存在").into_response()
        }
    }
}

/// 处理目录列表请求
async fn handle_list(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
    Query(root_params): Query<RootQuery>,
) -> Result<Json<Vec<FileInfo>>, StatusCode> {
    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let path = validate_and_resolve_path(&root_path, &params.path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !path.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut files = Vec::new();

    let entries = fs::read_dir(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for entry in entries {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let file_path = entry.path();
        let metadata = entry.metadata().ok();

        let name = entry
            .file_name()
            .to_str()
            .unwrap_or("")
            .to_string();

        let relative_path = pathdiff::diff_paths(&file_path, &root_path)
            .unwrap_or_else(|| file_path.clone());
        let relative_path_str = relative_path
            .to_str()
            .unwrap_or("")
            .to_string();

        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let mod_time = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| format!("{}", humantime::format_rfc3339_seconds(t)))
            .unwrap_or_default();

        let extension = if is_dir {
            None
        } else {
            file_path.extension().and_then(|e| e.to_str()).map(|s| s.to_string())
        };

        files.push(FileInfo {
            name,
            path: relative_path_str,
            is_dir,
            size,
            mod_time,
            extension,
        });
    }

    Ok(Json(files))
}

/// 处理文件查看请求
async fn handle_view(
    State(state): State<AppState>,
    Query(params): Query<FileQuery>,
    Query(root_params): Query<RootQuery>,
) -> Result<Json<FileViewResponse>, StatusCode> {
    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let path = validate_and_resolve_path(&root_path, &params.path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !path.is_file() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let metadata = fs::metadata(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = metadata.len();
    let total_lines = scanner::count_lines(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let (lines, page, total_pages, is_partial) = if file_size >= LARGE_FILE_THRESHOLD {
        // 大文件：分页读取
        let total_pages = ((total_lines as usize + LINES_PER_PAGE - 1) / LINES_PER_PAGE) as u32;
        let page = params.page.min(total_pages).max(1);
        let start_line = ((page - 1) * LINES_PER_PAGE as u32) as usize + 1;

        let lines = scanner::read_lines(&path, start_line, LINES_PER_PAGE)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        (lines, page, total_pages, true)
    } else {
        // 小文件：一次性读取所有行
        let lines = scanner::read_lines(&path, 1, total_lines as usize)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        (lines, 1, 1, false)
    };

    Ok(Json(FileViewResponse {
        name: file_name,
        path: params.path,
        size: file_size,
        total_lines,
        lines,
        page,
        total_pages,
        is_partial,
    }))
}

/// 处理搜索请求
async fn handle_search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
    Query(root_params): Query<RootQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let path = validate_and_resolve_path(&root_path, &params.path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !path.is_file() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let metadata = fs::metadata(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = metadata.len();
    let total_lines = scanner::count_lines(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query = params.q.to_lowercase();
    let mut results = Vec::new();

    let _total_pages = if file_size >= LARGE_FILE_THRESHOLD {
        ((total_lines as usize + LINES_PER_PAGE - 1) / LINES_PER_PAGE) as u32
    } else {
        1
    };

    // 使用扫描器逐行读取并搜索
    let file = fs::File::open(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut scanner = scanner::LineScanner::new(file);
    let mut line_number = 0u64;

    while let Some(line) = scanner.read_line().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        line_number += 1;

        if line.to_lowercase().contains(&query) {
            let page = if file_size >= LARGE_FILE_THRESHOLD {
                ((line_number as usize - 1) / LINES_PER_PAGE) as u32 + 1
            } else {
                1
            };

            results.push(SearchResult {
                line_number,
                page,
                line: line.to_string(),
            });
        }
    }

    Ok(Json(results))
}

/// 处理根目录列表请求
async fn handle_roots(State(state): State<AppState>) -> Json<Vec<config::RootDirConfig>> {
    Json(state.config.root_dirs.clone())
}

/// 处理保存文件请求
async fn handle_save(
    State(state): State<AppState>,
    Query(root_params): Query<RootQuery>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<SuccessResponse>, StatusCode> {
    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let path = validate_and_resolve_path(&root_path, &req.path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // 确保不是目录
    if path.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 写入文件
    fs::write(&path, &req.content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "文件保存成功".to_string(),
    }))
}

/// 处理删除文件请求
async fn handle_delete(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Query(root_params): Query<RootQuery>,
) -> Result<Json<SuccessResponse>, StatusCode> {
    let path = params.get("path").ok_or(StatusCode::BAD_REQUEST)?;

    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let full_path = validate_and_resolve_path(&root_path, path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // 确保不是目录
    if full_path.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 删除文件
    fs::remove_file(&full_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "文件删除成功".to_string(),
    }))
}

/// 处理创建文件请求
async fn handle_create(
    State(state): State<AppState>,
    Query(root_params): Query<RootQuery>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<SuccessResponse>, StatusCode> {
    if req.name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let dir_path = validate_and_resolve_path(&root_path, &req.path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let full_path = dir_path.join(&req.name);

    // 再次检查路径是否在根目录内
    let full_path_canonical = fs::canonicalize(&full_path)
        .unwrap_or_else(|_| full_path.clone());
    let root_path_canonical = fs::canonicalize(&root_path)
        .unwrap_or_else(|_| root_path.clone());

    if !full_path_canonical.starts_with(&root_path_canonical) {
        return Err(StatusCode::FORBIDDEN);
    }

    // 检查文件是否已存在
    if full_path.exists() {
        return Err(StatusCode::CONFLICT);
    }

    // 创建空文件
    fs::write(&full_path, "").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "文件创建成功".to_string(),
    }))
}

/// 处理创建目录请求
async fn handle_create_dir(
    State(state): State<AppState>,
    Query(root_params): Query<RootQuery>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<SuccessResponse>, StatusCode> {
    if req.name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let root_index = get_root_index_from_query(&root_params);
    let root_path = get_root_path(&state, root_index);

    let dir_path = validate_and_resolve_path(&root_path, &req.path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let full_path = dir_path.join(&req.name);

    // 再次检查路径是否在根目录内
    let full_path_canonical = fs::canonicalize(&full_path)
        .unwrap_or_else(|_| full_path.clone());
    let root_path_canonical = fs::canonicalize(&root_path)
        .unwrap_or_else(|_| root_path.clone());

    if !full_path_canonical.starts_with(&root_path_canonical) {
        return Err(StatusCode::FORBIDDEN);
    }

    // 检查目录是否已存在
    if full_path.exists() {
        return Err(StatusCode::CONFLICT);
    }

    // 创建目录
    fs::create_dir_all(&full_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "目录创建成功".to_string(),
    }))
}

/// 验证并解析路径，防止目录遍历攻击
fn validate_and_resolve_path(
    root_path: &Path,
    requested_path: &str,
) -> io::Result<std::path::PathBuf> {
    // 清理路径
    let clean_path = Path::new(requested_path).clean();

    // 检查是否包含非法的父目录引用
    let has_parent_refs = clean_path.components().any(|c| c == Component::ParentDir);
    if has_parent_refs {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "非法路径：包含父目录引用",
        ));
    }

    // 构建完整路径
    let full_path = if requested_path == "/" || requested_path == "" || requested_path == "." {
        root_path.to_path_buf()
    } else {
        root_path.join(&clean_path)
    };

    // 规范化路径（如果路径存在）
    let normalized = if full_path.exists() {
        fs::canonicalize(&full_path)?
    } else {
        full_path.clone()
    };

    // 确保路径在根目录内
    let normalized_str = normalized.to_str().unwrap_or("");
    let root_str = root_path.to_str().unwrap_or("");

    if !normalized_str.starts_with(root_str) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "访问被拒绝：路径超出根目录",
        ));
    }

    Ok(normalized)
}
