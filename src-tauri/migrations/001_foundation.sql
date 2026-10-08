CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY CHECK (version > 0),
    name TEXT NOT NULL,
    checksum TEXT NOT NULL CHECK (length(checksum) = 64),
    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE TABLE preferences (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT NOT NULL CHECK (theme IN ('light', 'dark', 'system')),
    locale TEXT NOT NULL CHECK (locale = 'tr'),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
INSERT INTO preferences (id, theme, locale) VALUES (1, 'system', 'tr');
