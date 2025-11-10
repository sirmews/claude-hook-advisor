//! Command history tracking with SQLite

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Represents a single command execution in history
#[derive(Debug, Clone)]
pub struct CommandRecord {
    pub timestamp: String,
    pub session_id: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub cwd: Option<String>,
    pub was_replaced: bool,
    pub original_command: Option<String>,
}

/// Initialize the command history database
pub fn init_database(db_path: &PathBuf) -> Result<Connection> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create history database directory")?;
    }

    let conn = Connection::open(db_path)
        .context("Failed to open history database")?;

    // Create table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS commands (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            session_id TEXT NOT NULL,
            command TEXT NOT NULL,
            exit_code INTEGER,
            cwd TEXT,
            was_replaced INTEGER NOT NULL DEFAULT 0,
            original_command TEXT
        )",
        [],
    )
    .context("Failed to create commands table")?;

    // Create indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON commands(timestamp)",
        [],
    )
    .context("Failed to create timestamp index")?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session ON commands(session_id)",
        [],
    )
    .context("Failed to create session_id index")?;

    Ok(conn)
}

/// Log a command to the history database
pub fn log_command(conn: &Connection, record: &CommandRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO commands (timestamp, session_id, command, exit_code, cwd, was_replaced, original_command)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            record.timestamp,
            record.session_id,
            record.command,
            record.exit_code,
            record.cwd,
            record.was_replaced as i32,
            record.original_command,
        ],
    )
    .context("Failed to insert command into history")?;

    Ok(())
}

/// Query options for retrieving command history
#[derive(Debug, Default)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
    pub session_id: Option<String>,
    pub failures_only: bool,
    pub command_pattern: Option<String>,
}

/// Retrieve command history with optional filters
pub fn query_history(conn: &Connection, query: &HistoryQuery) -> Result<Vec<CommandRecord>> {
    let mut sql = String::from(
        "SELECT timestamp, session_id, command, exit_code, cwd, was_replaced, original_command
         FROM commands WHERE 1=1"
    );

    // Build query based on filters
    if let Some(ref session_id) = query.session_id {
        sql.push_str(&format!(" AND session_id = '{}'", session_id));
    }

    if query.failures_only {
        sql.push_str(" AND exit_code != 0");
    }

    if let Some(ref pattern) = query.command_pattern {
        sql.push_str(&format!(" AND command LIKE '%{}%'", pattern));
    }

    sql.push_str(" ORDER BY timestamp DESC");

    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    let mut stmt = conn.prepare(&sql)
        .context("Failed to prepare query")?;

    let records = stmt.query_map([], |row| {
        Ok(CommandRecord {
            timestamp: row.get(0)?,
            session_id: row.get(1)?,
            command: row.get(2)?,
            exit_code: row.get(3)?,
            cwd: row.get(4)?,
            was_replaced: row.get::<_, i32>(5)? != 0,
            original_command: row.get(6)?,
        })
    })
    .context("Failed to execute query")?;

    let mut results = Vec::new();
    for record in records {
        results.push(record.context("Failed to parse command record")?);
    }

    Ok(results)
}

/// Create a command record from hook data
pub fn create_record(
    session_id: &str,
    command: &str,
    exit_code: Option<i32>,
    cwd: Option<&str>,
    was_replaced: bool,
    original_command: Option<&str>,
) -> CommandRecord {
    CommandRecord {
        timestamp: Utc::now().to_rfc3339(),
        session_id: session_id.to_string(),
        command: command.to_string(),
        exit_code,
        cwd: cwd.map(|s| s.to_string()),
        was_replaced,
        original_command: original_command.map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_initialization() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        let conn = init_database(&db_path).unwrap();

        // Verify table exists
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='commands'").unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists);
    }

    #[test]
    fn test_log_and_query_command() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let conn = init_database(&db_path).unwrap();

        // Log a command
        let record = create_record(
            "test-session",
            "npm install",
            Some(0),
            Some("/home/user/project"),
            false,
            None,
        );
        log_command(&conn, &record).unwrap();

        // Query it back
        let query = HistoryQuery {
            limit: Some(10),
            ..Default::default()
        };
        let results = query_history(&conn, &query).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "npm install");
        assert_eq!(results[0].session_id, "test-session");
        assert_eq!(results[0].exit_code, Some(0));
    }

    #[test]
    fn test_query_with_session_filter() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let conn = init_database(&db_path).unwrap();

        // Log commands from different sessions
        log_command(&conn, &create_record("session-1", "npm install", Some(0), None, false, None)).unwrap();
        log_command(&conn, &create_record("session-2", "yarn build", Some(0), None, false, None)).unwrap();
        log_command(&conn, &create_record("session-1", "npm test", Some(1), None, false, None)).unwrap();

        // Query session-1 only
        let query = HistoryQuery {
            session_id: Some("session-1".to_string()),
            ..Default::default()
        };
        let results = query_history(&conn, &query).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.session_id == "session-1"));
    }

    #[test]
    fn test_query_failures_only() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let conn = init_database(&db_path).unwrap();

        // Log successful and failed commands
        log_command(&conn, &create_record("session-1", "npm install", Some(0), None, false, None)).unwrap();
        log_command(&conn, &create_record("session-1", "npm test", Some(1), None, false, None)).unwrap();
        log_command(&conn, &create_record("session-1", "npm build", Some(2), None, false, None)).unwrap();

        // Query failures only
        let query = HistoryQuery {
            failures_only: true,
            ..Default::default()
        };
        let results = query_history(&conn, &query).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.exit_code != Some(0)));
    }

    #[test]
    fn test_query_with_command_pattern() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let conn = init_database(&db_path).unwrap();

        // Log various commands
        log_command(&conn, &create_record("session-1", "git status", Some(0), None, false, None)).unwrap();
        log_command(&conn, &create_record("session-1", "git commit", Some(0), None, false, None)).unwrap();
        log_command(&conn, &create_record("session-1", "npm install", Some(0), None, false, None)).unwrap();

        // Query for git commands only
        let query = HistoryQuery {
            command_pattern: Some("git".to_string()),
            ..Default::default()
        };
        let results = query_history(&conn, &query).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.command.contains("git")));
    }

    #[test]
    fn test_command_replacement_tracking() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let conn = init_database(&db_path).unwrap();

        // Log a replaced command
        let record = create_record(
            "test-session",
            "bun install",
            Some(0),
            None,
            true,
            Some("npm install"),
        );
        log_command(&conn, &record).unwrap();

        // Query it back
        let query = HistoryQuery::default();
        let results = query_history(&conn, &query).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "bun install");
        assert!(results[0].was_replaced);
        assert_eq!(results[0].original_command, Some("npm install".to_string()));
    }
}
