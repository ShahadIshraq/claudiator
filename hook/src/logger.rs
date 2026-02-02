use std::fs;
use std::io::Write;
use std::path::Path;

pub fn log_error(message: &str) {
    let Some(home_dir) = dirs::home_dir() else {
        return;
    };

    let log_path = home_dir.join(".claude/claudiator/error.log");
    log_error_to_path(&log_path, message);
}

fn log_error_to_path(path: &Path, message: &str) {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let log_line = format!("[{}] {}\n", timestamp, message);

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut file = match fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let _ = file.write_all(log_line.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;

    #[test]
    fn test_log_error_to_path_appends_line() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        log_error_to_path(&log_path, "test message");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("] test message\n"));
        assert!(content.starts_with('['));
    }

    #[test]
    fn test_log_error_to_path_format() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        log_error_to_path(&log_path, "error occurred");

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let line = lines[0];
        assert!(line.starts_with('['));
        assert!(line.contains("] error occurred"));

        let timestamp_end = line.find(']').unwrap();
        let timestamp = &line[1..timestamp_end];
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
    }

    #[test]
    fn test_log_error_to_path_multiple_calls() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        log_error_to_path(&log_path, "first error");
        log_error_to_path(&log_path, "second error");
        log_error_to_path(&log_path, "third error");

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);

        assert!(lines[0].contains("] first error"));
        assert!(lines[1].contains("] second error"));
        assert!(lines[2].contains("] third error"));
    }

    #[test]
    fn test_log_error_to_path_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("nested/dir/test.log");

        log_error_to_path(&log_path, "test message");

        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("] test message\n"));
    }
}
