CREATE TABLE IF NOT EXISTS auth_user_roles (
                user_id TEXT NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
                role_id TEXT NOT NULL REFERENCES auth_roles(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                PRIMARY KEY (user_id, role_id)
            )
