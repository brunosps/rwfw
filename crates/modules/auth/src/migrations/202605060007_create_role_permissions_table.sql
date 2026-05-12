CREATE TABLE IF NOT EXISTS auth.role_permissions (
                role_id UUID NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
                permission_id UUID NOT NULL REFERENCES auth.permissions(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (role_id, permission_id)
            )
