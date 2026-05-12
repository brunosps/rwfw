CREATE TABLE IF NOT EXISTS auth.oauth_login_states (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                provider VARCHAR(64) NOT NULL,
                state VARCHAR(255) NOT NULL UNIQUE,
                nonce VARCHAR(255) NOT NULL,
                pkce_verifier VARCHAR(255) NOT NULL,
                return_to TEXT,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
