DROP INDEX IF EXISTS idx_dns_zones_team_id;
ALTER TABLE dns_zones DROP COLUMN team_id;
