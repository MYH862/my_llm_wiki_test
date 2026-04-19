ALTER TABLE refresh_tokens ADD COLUMN IF NOT EXISTS jti VARCHAR(255);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_jti ON refresh_tokens(jti);
