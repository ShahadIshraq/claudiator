//! Raw event logging — appends the verbatim stdin JSON to a local JSONL file.
//!
//! When enabled, every event received from Claude Code is appended to the
//! configured file *before* any parsing or field trimming. This preserves the
//! full, unmodified event stream for offline inspection and schema-change
//! analysis.
//!
//! Errors are logged but never propagated — raw logging is best-effort and
//! must never disrupt the hook pipeline.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::Path;

use crate::logger::log_error;

/// Append `raw_json` as a single line to the JSONL file at `path`.
///
/// The entry is `raw_json` stripped of leading/trailing whitespace followed
/// by a newline, making the file valid JSONL (one JSON object per line).
///
/// The file and any missing parent directories are created automatically.
/// Errors are logged but not returned.
pub fn append_raw_event(path: &str, raw_json: &str) {
    let p = Path::new(path);

    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = fs::create_dir_all(parent) {
                log_error(&format!(
                    "raw_log: failed to create directory {}: {e}",
                    parent.display()
                ));
                return;
            }
        }
    }

    match OpenOptions::new().create(true).append(true).open(p) {
        Ok(mut f) => {
            let line = format!("{}\n", raw_json.trim());
            if let Err(e) = f.write_all(line.as_bytes()) {
                log_error(&format!("raw_log: failed to write to {path}: {e}"));
            }
        }
        Err(e) => {
            log_error(&format!("raw_log: failed to open {path}: {e}"));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_append_creates_file_and_writes_line() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("events.jsonl");
        let path_str = path.to_str().unwrap();

        append_raw_event(path_str, r#"{"session_id":"s1","hook_event_name":"Stop"}"#);

        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(
            contents,
            "{\"session_id\":\"s1\",\"hook_event_name\":\"Stop\"}\n"
        );
    }

    #[test]
    fn test_append_multiple_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("events.jsonl");
        let path_str = path.to_str().unwrap();

        append_raw_event(path_str, r#"{"session_id":"s1","hook_event_name":"A"}"#);
        append_raw_event(path_str, r#"{"session_id":"s1","hook_event_name":"B"}"#);

        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"A\""));
        assert!(lines[1].contains("\"B\""));
    }

    #[test]
    fn test_append_strips_surrounding_whitespace() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("events.jsonl");
        let path_str = path.to_str().unwrap();

        append_raw_event(
            path_str,
            "  {\"session_id\":\"s1\",\"hook_event_name\":\"X\"}  \n",
        );

        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(
            contents,
            "{\"session_id\":\"s1\",\"hook_event_name\":\"X\"}\n"
        );
    }

    #[test]
    fn test_append_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a").join("b").join("events.jsonl");
        let path_str = path.to_str().unwrap();

        append_raw_event(path_str, r#"{"session_id":"s1","hook_event_name":"Y"}"#);

        assert!(path.exists());
    }
}
