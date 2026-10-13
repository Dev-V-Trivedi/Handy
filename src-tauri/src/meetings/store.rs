//! SQLite storage for meeting transcripts and summaries.
//!
//! Only text is stored. Meeting audio is never written to disk.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use specta::Type;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Type)]
pub struct MeetingRow {
    pub id: i64,
    pub title: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub has_summary: bool,
    pub segment_count: i64,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct MeetingSegment {
    /// `you` (microphone) or `others` (computer audio).
    pub track: String,
    pub start_ms: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct MeetingDetail {
    pub id: i64,
    pub title: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    /// Summary as a JSON string (see `summary.rs` for the shape), if made.
    pub summary: Option<String>,
    pub segments: Vec<MeetingSegment>,
}

pub struct MeetingStore {
    conn: Mutex<Connection>,
}

impl MeetingStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS meetings (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL,
                 title_is_default INTEGER NOT NULL DEFAULT 1,
                 started_at INTEGER NOT NULL,
                 ended_at INTEGER,
                 summary TEXT
             );
             CREATE TABLE IF NOT EXISTS segments (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 meeting_id INTEGER NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                 track TEXT NOT NULL,
                 start_ms INTEGER NOT NULL,
                 text TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_segments_meeting
                 ON segments(meeting_id, start_ms);",
        )?;
        conn.execute_batch("PRAGMA foreign_keys=ON")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.close_interrupted()?;
        Ok(store)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| anyhow::anyhow!("meeting database lock poisoned"))
    }

    /// A meeting still open at startup was cut short (crash, power loss).
    /// Close it at the time of its last segment so it shows up as finished.
    fn close_interrupted(&self) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE meetings
             SET ended_at = started_at + COALESCE(
                 (SELECT MAX(start_ms) / 1000 FROM segments WHERE meeting_id = meetings.id), 0)
             WHERE ended_at IS NULL",
            [],
        )?;
        Ok(())
    }

    pub fn create(&self, title: &str, is_default: bool, started_at: i64) -> Result<i64> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO meetings (title, title_is_default, started_at) VALUES (?1, ?2, ?3)",
            params![title, is_default as i64, started_at],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn add_segment(
        &self,
        meeting_id: i64,
        track: &str,
        start_ms: i64,
        text: &str,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO segments (meeting_id, track, start_ms, text) VALUES (?1, ?2, ?3, ?4)",
            params![meeting_id, track, start_ms, text],
        )?;
        Ok(())
    }

    pub fn finish(&self, id: i64, ended_at: i64) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE meetings SET ended_at = ?1 WHERE id = ?2",
            params![ended_at, id],
        )?;
        Ok(())
    }

    /// Save the summary JSON. If the user never renamed the meeting, adopt the
    /// title the model suggested.
    pub fn set_summary(
        &self,
        id: i64,
        summary_json: &str,
        suggested_title: Option<&str>,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE meetings SET summary = ?1 WHERE id = ?2",
            params![summary_json, id],
        )?;
        if let Some(title) = suggested_title.map(str::trim).filter(|t| !t.is_empty()) {
            conn.execute(
                "UPDATE meetings SET title = ?1 WHERE id = ?2 AND title_is_default = 1",
                params![title, id],
            )?;
        }
        Ok(())
    }

    pub fn rename(&self, id: i64, title: &str) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE meetings SET title = ?1, title_is_default = 0 WHERE id = ?2",
            params![title, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM segments WHERE meeting_id = ?1", params![id])?;
        conn.execute("DELETE FROM meetings WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete finished meetings that started before `cutoff` (unix seconds).
    /// Meetings still being recorded are never touched. Returns how many went.
    pub fn purge_older_than(&self, cutoff: i64) -> Result<usize> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM segments WHERE meeting_id IN
             (SELECT id FROM meetings WHERE started_at < ?1 AND ended_at IS NOT NULL)",
            params![cutoff],
        )?;
        let n = conn.execute(
            "DELETE FROM meetings WHERE started_at < ?1 AND ended_at IS NOT NULL",
            params![cutoff],
        )?;
        Ok(n)
    }

    pub fn list(&self) -> Result<Vec<MeetingRow>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT m.id, m.title, m.started_at, m.ended_at, m.summary IS NOT NULL,
                    (SELECT COUNT(*) FROM segments s WHERE s.meeting_id = m.id)
             FROM meetings m ORDER BY m.started_at DESC, m.id DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MeetingRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    has_summary: row.get::<_, i64>(4)? != 0,
                    segment_count: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get(&self, id: i64) -> Result<Option<MeetingDetail>> {
        let conn = self.lock()?;
        let head = conn
            .query_row(
                "SELECT id, title, started_at, ended_at, summary FROM meetings WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;
        let Some((id, title, started_at, ended_at, summary)) = head else {
            return Ok(None);
        };
        let mut stmt = conn.prepare(
            "SELECT track, start_ms, text FROM segments WHERE meeting_id = ?1
             ORDER BY start_ms, id",
        )?;
        let segments = stmt
            .query_map(params![id], |row| {
                Ok(MeetingSegment {
                    track: row.get(0)?,
                    start_ms: row.get(1)?,
                    text: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(Some(MeetingDetail {
            id,
            title,
            started_at,
            ended_at,
            summary,
            // Readers get conversation turns; raw chunks stay in the database.
            segments: super::tidy::merge_turns(segments),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (MeetingStore, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "oshabe-meetings-test-{}-{:?}.db",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_file(&path);
        (MeetingStore::open(&path).unwrap(), path)
    }

    #[test]
    fn roundtrip_and_rename_rules() {
        let (store, path) = temp_store();
        let id = store.create("Meeting 1", true, 1_000).unwrap();
        store.add_segment(id, "others", 5_000, "hello").unwrap();
        store.add_segment(id, "you", 1_000, "hi there").unwrap();
        store.finish(id, 1_060).unwrap();

        // Auto title adopts the model's suggestion.
        store
            .set_summary(id, "{\"summary\":\"x\"}", Some("Budget sync"))
            .unwrap();
        let got = store.get(id).unwrap().unwrap();
        assert_eq!(got.title, "Budget sync");
        assert_eq!(got.segments[0].text, "Hi there."); // ordered by time, tidied
        assert!(got.summary.is_some());

        // A user rename is never overwritten.
        store.rename(id, "Mine").unwrap();
        store.set_summary(id, "{}", Some("Other")).unwrap();
        assert_eq!(store.get(id).unwrap().unwrap().title, "Mine");

        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].segment_count, 2);
        assert!(list[0].has_summary);

        store.delete(id).unwrap();
        assert!(store.get(id).unwrap().is_none());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn purge_removes_only_old_finished_meetings() {
        let (store, path) = temp_store();
        let old = store.create("Old", true, 100).unwrap();
        store.add_segment(old, "you", 0, "x").unwrap();
        store.finish(old, 200).unwrap();
        let fresh = store.create("Fresh", true, 10_000).unwrap();
        store.finish(fresh, 10_100).unwrap();
        let running = store.create("Running", true, 50).unwrap(); // never finished

        assert_eq!(store.purge_older_than(5_000).unwrap(), 1);
        assert!(store.get(old).unwrap().is_none());
        assert!(store.get(fresh).unwrap().is_some());
        assert!(store.get(running).unwrap().is_some());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn interrupted_meetings_are_closed_on_open() {
        let (store, path) = temp_store();
        let id = store.create("Cut short", true, 2_000).unwrap();
        store.add_segment(id, "you", 90_000, "last words").unwrap();
        drop(store);
        let reopened = MeetingStore::open(&path).unwrap();
        let got = reopened.get(id).unwrap().unwrap();
        assert_eq!(got.ended_at, Some(2_090));
        let _ = std::fs::remove_file(path);
    }
}
