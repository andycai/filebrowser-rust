use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// 优化的行扫描器，用于高效读取大文件
pub struct LineScanner {
    reader: BufReader<File>,
    line: String,
}

impl LineScanner {
    /// 创建新的行扫描器
    pub fn new(file: File) -> Self {
        // 使用 64KB 缓冲区
        let reader = BufReader::with_capacity(64 * 1024, file);
        LineScanner {
            reader,
            line: String::new(),
        }
    }

    /// 读取下一行
    pub fn read_line(&mut self) -> io::Result<Option<&str>> {
        self.line.clear();
        let bytes_read = self.reader.read_line(&mut self.line)?;

        if bytes_read == 0 {
            Ok(None)
        } else {
            // 移除换行符
            if self.line.ends_with('\n') {
                self.line.pop();
                if self.line.ends_with('\r') {
                    self.line.pop();
                }
            }
            Ok(Some(&self.line))
        }
    }
}

/// 快速计算文件行数
pub fn count_lines<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);

    let mut count = 0u64;
    for line in reader.lines() {
        let _ = line?;
        count += 1;
    }

    Ok(count)
}

/// 读取指定范围内的行
pub fn read_lines<P: AsRef<Path>>(
    path: P,
    start_line: usize,
    count: usize,
) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);

    let mut lines = Vec::with_capacity(count);
    let mut current_line = 0;

    for line in reader.lines() {
        let line = line?;
        current_line += 1;

        if current_line >= start_line && current_line < start_line + count {
            lines.push(line);
        }

        if current_line >= start_line + count {
            break;
        }
    }

    Ok(lines)
}
