CREATE TABLE IF NOT EXISTS auth.user_roles (
                user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
                role_id UUID NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (user_id, role_id)
            )
