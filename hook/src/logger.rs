//! File-based logger with size-capped rotation.
//!
//! Log output goes to `~/.claude/claudiator/error.log`. Stderr is intentionally
//! avoided for normal operation: Claude Code surfaces stderr output to the user,
//! which would be noisy for routine events. Only the `test` subcommand writes
//! to stderr (on failure) because it is run interactively by the user.
//!
//! # Initialization
//!
//! Call [`init`] once at startup with the desired level and rotation settings.
//! If the log functions are called before `init`, a safe default config
//! (level = Error, 1 MiB, 2 backups) is used automatically.
//!
//! # Rotation
//!
//! When the log file exceeds `max_size_bytes`, it is renamed to `.1`, existing
//! `.1` becomes `.2`, and so on up to `max_backups`. The oldest backup is
//! deleted. If `max_backups` is 0 the file is simply truncated in place.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;

/// Log verbosity levels, ordered from least to most verbose.
///
/// The numeric values are meaningful: a level is active when it is less than
/// or equal to the configured maximum, which makes the comparison in [`log`]
/// a simple integer comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warn => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
            Self::Debug => write!(f, "DEBUG"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLogLevelError;

impl std::fmt::Display for ParseLogLevelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid log level")
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            _ => Err(ParseLogLevelError),
        }
    }
}

struct LogConfig {
    level: LogLevel,
    max_size_bytes: u64,
    max_backups: u32,
}

static LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new();

/// Initialize the logger.
///
/// Must be called once before any log helpers are used. Subsequent calls are
/// silently ignored (the `OnceLock` ensures the first write wins).
pub fn init(level: LogLevel, max_size_bytes: u64, max_backups: u32) {
    let _ = LOG_CONFIG.set(LogConfig {
        level,
        max_size_bytes,
        max_backups,
    });
}

fn get_config() -> &'static LogConfig {
    LOG_CONFIG.get_or_init(|| LogConfig {
        level: LogLevel::Error,
        max_size_bytes: 1_048_576,
        max_backups: 2,
    })
}

pub fn log_error(message: &str) {
    log(LogLevel::Error, message);
}

#[allow(dead_code)]
pub fn log_warn(message: &str) {
    log(LogLevel::Warn, message);
}

pub fn log_info(message: &str) {
    log(LogLevel::Info, message);
}

pub fn log_debug(message: &str) {
    log(LogLevel::Debug, message);
}

fn log(level: LogLevel, message: &str) {
    let config = get_config();
    if level > config.level {
        return;
    }

    let Some(home_dir) = dirs::home_dir() else {
        return;
    };

    let log_path = home_dir.join(".claude/claudiator/error.log");
    log_to_path(
        &log_path,
        level,
        message,
        config.max_size_bytes,
        config.max_backups,
    );
}

fn log_to_path(path: &Path, level: LogLevel, message: &str, max_size_bytes: u64, max_backups: u32) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    maybe_rotate(path, max_size_bytes, max_backups);

    let timestamp = chrono::Utc::now().to_rfc3339();
    let log_line = format!("[{timestamp}] [{level}] {message}\n");

    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let _ = file.write_all(log_line.as_bytes());
}

fn maybe_rotate(path: &Path, max_size_bytes: u64, max_backups: u32) {
    let Ok(metadata) = fs::metadata(path) else {
        return; // file doesn't exist yet, nothing to rotate
    };

    if metadata.len() < max_size_bytes {
        return; // fast path: file is under size limit
    }

    if max_backups == 0 {
        // Truncate the file
        let _ = fs::File::create(path);
        return;
    }

    // Delete the oldest backup if it exists
    let oldest = format!("{}.{max_backups}", path.display());
    let _ = fs::remove_file(&oldest);

    // Shift backups: .{i} -> .{i+1}, starting from the oldest
    for i in (1..max_backups).rev() {
        let from = format!("{}.{i}", path.display());
        let to = format!("{}.{}", path.display(), i + 1);
        let _ = fs::rename(&from, &to);
    }

    // Rename current log to .1
    let backup_1 = format!("{}.1", path.display());
    let _ = fs::rename(path, &backup_1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
    }

    #[test]
    fn test_log_level_from_str() {
        use std::str::FromStr;

        // Valid inputs - case insensitive
        let result = LogLevel::from_str("error");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Error);
        }

        let result = LogLevel::from_str("ERROR");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Error);
        }

        let result = LogLevel::from_str("Error");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Error);
        }

        let result = LogLevel::from_str("warn");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Warn);
        }

        let result = LogLevel::from_str("WARN");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Warn);
        }

        let result = LogLevel::from_str("info");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Info);
        }

        let result = LogLevel::from_str("INFO");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Info);
        }

        let result = LogLevel::from_str("debug");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Debug);
        }

        let result = LogLevel::from_str("DEBUG");
        assert!(result.is_ok());
        if let Ok(level) = result {
            assert_eq!(level, LogLevel::Debug);
        }

        // Invalid inputs
        assert!(LogLevel::from_str("invalid").is_err());
        assert!(LogLevel::from_str("").is_err());
        assert!(LogLevel::from_str("trace").is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
        assert_eq!(format!("{}", LogLevel::Warn), "WARN");
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
    }

    #[test]
    fn test_log_to_path_includes_level_tag() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        log_to_path(&log_path, LogLevel::Error, "test message", 1024, 2);

        let content = fs::read_to_string(&log_path);
        assert!(content.is_ok());
        let Ok(content) = content else { return };
        assert!(content.contains("[ERROR]"));
    }

    #[test]
    fn test_log_to_path_format() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        log_to_path(&log_path, LogLevel::Info, "test message", 1024, 2);

        let content = fs::read_to_string(&log_path);
        assert!(content.is_ok());
        let Ok(content) = content else { return };

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let line = lines[0];
        assert!(line.starts_with('['));
        assert!(line.contains("] [INFO] test message"));

        // Extract timestamp
        let timestamp_end = line.find(']');
        assert!(timestamp_end.is_some());
        let Some(timestamp_end) = timestamp_end else {
            return;
        };
        let timestamp = &line[1..timestamp_end];
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
    }

    #[test]
    fn test_log_to_path_appends() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        log_to_path(&log_path, LogLevel::Error, "first", 1024, 2);
        log_to_path(&log_path, LogLevel::Warn, "second", 1024, 2);
        log_to_path(&log_path, LogLevel::Info, "third", 1024, 2);

        let content = fs::read_to_string(&log_path);
        assert!(content.is_ok());
        let Ok(content) = content else { return };

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);

        assert!(lines[0].contains("[ERROR]"));
        assert!(lines[0].contains("first"));
        assert!(lines[1].contains("[WARN]"));
        assert!(lines[1].contains("second"));
        assert!(lines[2].contains("[INFO]"));
        assert!(lines[2].contains("third"));
    }

    #[test]
    fn test_log_to_path_creates_directory() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("nested/dir/test.log");

        log_to_path(&log_path, LogLevel::Error, "test message", 1024, 2);

        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path);
        assert!(content.is_ok());
        let Ok(content) = content else { return };
        assert!(content.contains("[ERROR]"));
        assert!(content.contains("test message"));
    }

    #[test]
    fn test_maybe_rotate_no_rotation_under_limit() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        let small_content = "small";
        let write_result = fs::write(&log_path, small_content);
        assert!(write_result.is_ok());

        maybe_rotate(&log_path, 100, 2);

        // File should still exist with same content
        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path);
        assert!(content.is_ok());
        let Ok(content) = content else { return };
        assert_eq!(content, small_content);

        // No backup should exist
        let backup_path = format!("{}.1", log_path.display());
        assert!(!Path::new(&backup_path).exists());
    }

    #[test]
    fn test_maybe_rotate_triggers_rotation() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        let large_content = "this is large content that exceeds limit";
        let write_result = fs::write(&log_path, large_content);
        assert!(write_result.is_ok());

        maybe_rotate(&log_path, 10, 2);

        // Original file should either not exist or be empty/smaller
        // (it gets renamed to .1)
        let backup_path = format!("{}.1", log_path.display());
        assert!(Path::new(&backup_path).exists());

        let backup_content = fs::read_to_string(&backup_path);
        assert!(backup_content.is_ok());
        let Ok(backup_content) = backup_content else {
            return;
        };
        assert_eq!(backup_content, large_content);
    }

    #[test]
    fn test_maybe_rotate_shifts_backups() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        // Create existing .1 backup
        let backup_1_path = format!("{}.1", log_path.display());
        let old_backup_content = "old backup content";
        let write_result = fs::write(&backup_1_path, old_backup_content);
        assert!(write_result.is_ok());

        // Create current log file at max size
        let current_content = "current log content that is large";
        let write_result = fs::write(&log_path, current_content);
        assert!(write_result.is_ok());

        maybe_rotate(&log_path, 10, 2);

        // .1 should have the latest content (from current log)
        let backup_1_content = fs::read_to_string(&backup_1_path);
        assert!(backup_1_content.is_ok());
        let Ok(backup_1_content) = backup_1_content else {
            return;
        };
        assert_eq!(backup_1_content, current_content);

        // .2 should have the old .1 content
        let backup_2_path = format!("{}.2", log_path.display());
        assert!(Path::new(&backup_2_path).exists());
        let backup_2_content = fs::read_to_string(&backup_2_path);
        assert!(backup_2_content.is_ok());
        let Ok(backup_2_content) = backup_2_content else {
            return;
        };
        assert_eq!(backup_2_content, old_backup_content);
    }

    #[test]
    fn test_maybe_rotate_zero_backups_truncates() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        let large_content = "this is large content";
        let write_result = fs::write(&log_path, large_content);
        assert!(write_result.is_ok());

        maybe_rotate(&log_path, 10, 0);

        // File should exist but be empty
        assert!(log_path.exists());
        let metadata = fs::metadata(&log_path);
        assert!(metadata.is_ok());
        let Ok(metadata) = metadata else { return };
        assert_eq!(metadata.len(), 0);

        // No backups should exist
        let backup_path = format!("{}.1", log_path.display());
        assert!(!Path::new(&backup_path).exists());
    }

    #[test]
    fn test_maybe_rotate_deletes_oldest() {
        let temp_dir = tempfile::tempdir();
        assert!(temp_dir.is_ok());
        let Ok(temp_dir) = temp_dir else { return };
        let log_path = temp_dir.path().join("test.log");

        // Create .1 and .2 backups
        let backup_1_path = format!("{}.1", log_path.display());
        let backup_1_content = "backup 1 content";
        let write_result = fs::write(&backup_1_path, backup_1_content);
        assert!(write_result.is_ok());

        let backup_2_path = format!("{}.2", log_path.display());
        let backup_2_content = "backup 2 content";
        let write_result = fs::write(&backup_2_path, backup_2_content);
        assert!(write_result.is_ok());

        // Create current log file at max size
        let current_content = "current log content that exceeds limit";
        let write_result = fs::write(&log_path, current_content);
        assert!(write_result.is_ok());

        maybe_rotate(&log_path, 10, 2);

        // .1 should have current content
        let new_backup_1_content = fs::read_to_string(&backup_1_path);
        assert!(new_backup_1_content.is_ok());
        let Ok(new_backup_1_content) = new_backup_1_content else {
            return;
        };
        assert_eq!(new_backup_1_content, current_content);

        // .2 should have old .1 content
        let new_backup_2_content = fs::read_to_string(&backup_2_path);
        assert!(new_backup_2_content.is_ok());
        let Ok(new_backup_2_content) = new_backup_2_content else {
            return;
        };
        assert_eq!(new_backup_2_content, backup_1_content);

        // Old .2 was deleted, so we should only have 2 backups total
        let backup_3_path = format!("{}.3", log_path.display());
        assert!(!Path::new(&backup_3_path).exists());
    }
}
