CREATE TABLE IF NOT EXISTS posts
(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    title              TEXT NOT NULL,
    content            TEXT NOT NULL,
    create_time        TEXT NOT NULL,
    last_modified_time TEXT NOT NULL
)
