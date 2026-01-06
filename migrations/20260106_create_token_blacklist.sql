-- Token blacklist table (Idempotent migration)
-- Can be run multiple times without error

CREATE TABLE IF NOT EXISTS auth_token_blacklist (
    id SERIAL PRIMARY KEY,
    token_jti TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auth_token_blacklist_jti ON auth_token_blacklist(token_jti);
CREATE INDEX IF NOT EXISTS idx_auth_token_blacklist_expires_at ON auth_token_blacklist(expires_at);
