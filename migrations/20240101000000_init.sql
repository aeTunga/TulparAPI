CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alias TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    language TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_collections_alias ON collections(alias);

INSERT INTO collections (alias, name, file_path) 
VALUES ('rubaiyat', 'Rubaiyat of Omar Khayyam', 'storage/collections/rubaiyat.json.lz4') 
ON CONFLICT(alias) DO UPDATE SET name=excluded.name, file_path=excluded.file_path;
