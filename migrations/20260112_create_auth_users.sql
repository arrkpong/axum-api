-- Users table for authentication (Idempotent migration)
-- Can be run multiple times without error

-- Create table if not exists
CREATE TABLE IF NOT EXISTS auth_users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add columns if not exists (for existing tables)
ALTER TABLE auth_users ADD COLUMN IF NOT EXISTS email VARCHAR(255) UNIQUE;
ALTER TABLE auth_users ADD COLUMN IF NOT EXISTS phone VARCHAR(50);

-- Create indexes if not exists
CREATE INDEX IF NOT EXISTS idx_auth_users_username ON auth_users(username);
CREATE INDEX IF NOT EXISTS idx_auth_users_email ON auth_users(email);
