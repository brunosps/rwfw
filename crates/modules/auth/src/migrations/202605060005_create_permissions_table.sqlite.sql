CREATE TABLE IF NOT EXISTS auth_permissions (
                id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            )
