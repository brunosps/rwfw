CREATE TABLE IF NOT EXISTS auth_role_permissions (
                role_id UUID NOT NULL REFERENCES auth_roles(id) ON DELETE CASCADE,
                permission_id UUID NOT NULL REFERENCES auth_permissions(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (role_id, permission_id)
            )
