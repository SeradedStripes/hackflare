ALTER TABLE dns_zones ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE SET NULL;
CREATE INDEX idx_dns_zones_team_id ON dns_zones(team_id);
