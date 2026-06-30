ALTER TABLE api_keys ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE CASCADE;
CREATE INDEX idx_api_keys_team_id ON api_keys(team_id);
