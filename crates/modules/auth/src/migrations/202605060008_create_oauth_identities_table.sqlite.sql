CREATE TABLE IF NOT EXISTS auth_oauth_identities (
                id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
                user_id TEXT NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
                provider TEXT NOT NULL,
                issuer TEXT NOT NULL,
                subject TEXT NOT NULL,
                email TEXT,
                email_verified INTEGER NOT NULL DEFAULT 0,
                claims TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                UNIQUE(provider, subject),
                UNIQUE(issuer, subject)
            )
