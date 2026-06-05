CREATE TABLE IF NOT EXISTS auth_role_permissions (
                role_id TEXT NOT NULL REFERENCES auth_roles(id) ON DELETE CASCADE,
                permission_id TEXT NOT NULL REFERENCES auth_permissions(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                PRIMARY KEY (role_id, permission_id)
            )
