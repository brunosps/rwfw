CREATE TABLE IF NOT EXISTS auth_user_roles (
                user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
                role_id UUID NOT NULL REFERENCES auth_roles(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (user_id, role_id)
            )
