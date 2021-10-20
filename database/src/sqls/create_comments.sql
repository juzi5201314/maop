CREATE TABLE IF NOT EXISTS comments
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id     INTEGER NOT NULL,
    content     TEXT    NOT NULL,
    create_time TEXT    NOT NULL,
    email       TEXT    NOT NULL,
    nickname    TEXT    NOT NULL,
    parent_id   INTEGER,
    deleted     INTEGER NOT NULL
)
