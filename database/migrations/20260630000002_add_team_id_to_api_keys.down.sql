DROP INDEX IF EXISTS idx_api_keys_team_id;
ALTER TABLE api_keys DROP COLUMN team_id;
