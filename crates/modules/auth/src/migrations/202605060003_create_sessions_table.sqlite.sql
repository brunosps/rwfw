CREATE TABLE IF NOT EXISTS auth_sessions (
                id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
                user_id TEXT NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
                token TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            )
