use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

use super::models::{Bind, Category, Track};

const MIGRATIONS: &str = "
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    icon_name TEXT
);

CREATE TABLE IF NOT EXISTS tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    volume REAL NOT NULL DEFAULT 1.0,
    source TEXT NOT NULL DEFAULT 'local',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS binds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    keyval INTEGER NOT NULL,
    modifiers INTEGER NOT NULL DEFAULT 0,
    UNIQUE(keyval, modifiers)
);

INSERT OR IGNORE INTO categories (name, icon_name) VALUES
    ('Мемы', 'face-laugh'),
    ('Игры', 'gamepad'),
    ('Аниме', 'tv'),
    ('Музыка', 'music-note'),
    ('Природа', 'tree'),
    ('Разное', 'folder');
";

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Не удалось открыть БД: {}", path.display()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(MIGRATIONS)?;
        Ok(Self { conn })
    }

    pub fn open_default() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Не удалось определить директорию данных")?
            .join("soundpaad");
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("soundpaad.db");
        Self::open(&db_path)
    }

    // === Tracks ===

    pub fn add_track(&self, track: &Track) -> Result<i64> {
        // Автоудаление дубликата по file_path
        self.conn.execute(
            "DELETE FROM tracks WHERE file_path = ?1 AND id != (SELECT id FROM tracks WHERE file_path = ?1 LIMIT 1)",
            rusqlite::params![track.file_path],
        ).ok();

        self.conn.execute(
            "INSERT OR REPLACE INTO tracks (name, file_path, category_id, volume, source) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![track.name, track.file_path, track.category_id, track.volume, track.source],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_tracks(&self) -> Result<Vec<Track>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path, category_id, volume, source FROM tracks ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Track {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                file_path: row.get(2)?,
                category_id: row.get(3)?,
                volume: row.get(4)?,
                source: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_tracks_by_category(&self, category_id: i64) -> Result<Vec<Track>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path, category_id, volume, source FROM tracks WHERE category_id = ?1 ORDER BY name",
        )?;
        let rows = stmt.query_map([category_id], |row| {
            Ok(Track {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                file_path: row.get(2)?,
                category_id: row.get(3)?,
                volume: row.get(4)?,
                source: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete_track(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM tracks WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_track_volume(&self, id: i64, volume: f64) -> Result<()> {
        self.conn.execute("UPDATE tracks SET volume = ?1 WHERE id = ?2", rusqlite::params![volume, id])?;
        Ok(())
    }

    // === Binds ===

    pub fn add_bind(&self, bind: &Bind) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO binds (track_id, keyval, modifiers) VALUES (?1, ?2, ?3)",
            rusqlite::params![bind.track_id, bind.keyval, bind.modifiers],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_binds(&self) -> Result<Vec<Bind>> {
        let mut stmt = self.conn.prepare("SELECT id, track_id, keyval, modifiers FROM binds")?;
        let rows = stmt.query_map([], |row| {
            Ok(Bind {
                id: Some(row.get(0)?),
                track_id: row.get(1)?,
                keyval: row.get(2)?,
                modifiers: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_bind_by_key(&self, keyval: u32, modifiers: u32) -> Result<Option<Bind>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, track_id, keyval, modifiers FROM binds WHERE keyval = ?1 AND modifiers = ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![keyval, modifiers], |row| {
            Ok(Bind {
                id: Some(row.get(0)?),
                track_id: row.get(1)?,
                keyval: row.get(2)?,
                modifiers: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map(|v| v.into_iter().next())
            .map_err(Into::into)
    }

    pub fn delete_bind(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM binds WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn delete_bind_by_track(&self, track_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM binds WHERE track_id = ?1", [track_id])?;
        Ok(())
    }

    // === Categories ===

    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.conn.prepare("SELECT id, name, icon_name FROM categories ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon_name: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn add_category(&self, name: &str, icon_name: Option<&str>) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO categories (name, icon_name) VALUES (?1, ?2)",
            rusqlite::params![name, icon_name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
}
