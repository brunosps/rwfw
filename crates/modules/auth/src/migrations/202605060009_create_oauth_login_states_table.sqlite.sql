CREATE TABLE IF NOT EXISTS auth_oauth_login_states (
                id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
                provider TEXT NOT NULL,
                state TEXT NOT NULL UNIQUE,
                nonce TEXT NOT NULL,
                pkce_verifier TEXT NOT NULL,
                return_to TEXT,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            )
