CREATE TABLE IF NOT EXISTS auth.oauth_identities (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
                provider VARCHAR(64) NOT NULL,
                issuer TEXT NOT NULL,
                subject TEXT NOT NULL,
                email VARCHAR(255),
                email_verified BOOLEAN NOT NULL DEFAULT FALSE,
                claims JSONB NOT NULL DEFAULT '{}'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(provider, subject),
                UNIQUE(issuer, subject)
            )
