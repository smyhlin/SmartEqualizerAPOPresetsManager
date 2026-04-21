use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Local};
use serde::Serialize;

use crate::state::{AppError, APP_FOLDER_NAME};

const LOGS_DIR_NAME: &str = "logs";
const LOG_FILE_NAME: &str = "application.log";
const LOG_TAIL_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSnapshot {
    pub log_path: String,
    pub content: String,
    pub exists: bool,
}

pub(crate) fn append_log_line(level: &str, message: impl AsRef<str>) {
    let Ok(path) = log_file_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let timestamp = format_timestamp(SystemTime::now());
    let line = format!(
        "[{timestamp}] [{}] {}\r\n",
        level.trim().to_uppercase(),
        message.as_ref().trim()
    );

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
    }
}

pub(crate) fn read_log_snapshot() -> Result<LogSnapshot, AppError> {
    let path = log_file_path()?;

    if !path.exists() {
        return Ok(LogSnapshot {
            log_path: path.to_string_lossy().into_owned(),
            content: "No logs have been written yet.".to_string(),
            exists: false,
        });
    }

    let content = match read_tail(&path) {
        Ok(content) => content,
        Err(error) => format!("Unable to read the log file:\n{error}"),
    };

    Ok(LogSnapshot {
        log_path: path.to_string_lossy().into_owned(),
        content,
        exists: true,
    })
}

pub(crate) fn log_folder_path() -> Result<PathBuf, AppError> {
    let path = log_file_path()?;
    let folder = path.parent().ok_or_else(|| {
        AppError::Message(format!(
            "Unable to resolve the log folder from '{}'.",
            path.display()
        ))
    })?;
    fs::create_dir_all(folder)?;
    Ok(folder.to_path_buf())
}

fn log_file_path() -> Result<PathBuf, AppError> {
    let base_dir = dirs::config_dir()
        .ok_or(AppError::AppDataUnavailable)?
        .join(APP_FOLDER_NAME);
    Ok(base_dir.join(LOGS_DIR_NAME).join(LOG_FILE_NAME))
}

fn format_timestamp(timestamp: SystemTime) -> String {
    let datetime: DateTime<Local> = timestamp.into();
    datetime.format("%Y-%m-%d %H:%M:%S %:z").to_string()
}

fn read_tail(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path)?;
    if bytes.len() <= LOG_TAIL_BYTES {
        return Ok(String::from_utf8_lossy(&bytes).into_owned());
    }

    Ok(String::from_utf8_lossy(&bytes[bytes.len() - LOG_TAIL_BYTES..]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_should_use_readable_local_shape() {
        let formatted = format_timestamp(SystemTime::UNIX_EPOCH);
        let bytes = formatted.as_bytes();
        let offset = &formatted[formatted.len() - 6..];

        assert!(bytes.len() >= 25, "unexpected timestamp length: {formatted}");
        assert_eq!(bytes[4], b'-');
        assert_eq!(bytes[7], b'-');
        assert_eq!(bytes[10], b' ');
        assert_eq!(bytes[13], b':');
        assert_eq!(bytes[16], b':');
        assert!(offset.starts_with('+') || offset.starts_with('-'));
        assert_eq!(offset.as_bytes()[3], b':');
    }
}
