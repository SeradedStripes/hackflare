use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query, query_as};
use uuid::Uuid;

use crate::models::db::{Team, TeamMember};

#[derive(Clone)]
pub(crate) struct TeamsService {
    db: PgPool,
}

fn name_to_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

impl TeamsService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn create(&self, name: &str, created_by: &str) -> Result<Team> {
        let slug = name_to_slug(name);
        let team = query_as::<_, Team>(
            r#"
            INSERT INTO teams (name, slug, created_by)
            VALUES ($1, $2, $3)
            RETURNING id, name, slug, created_by, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(&slug)
        .bind(created_by)
        .fetch_one(&self.db)
        .await?;

        // Add creator as owner
        query(
            r#"
            INSERT INTO team_members (team_id, user_id, role)
            VALUES ($1, $2, 'owner')
            "#,
        )
        .bind(team.id)
        .bind(created_by)
        .execute(&self.db)
        .await?;

        Ok(team)
    }

    pub(crate) async fn list_for_user(&self, user_id: &str) -> Result<Vec<Team>> {
        let teams = query_as::<_, Team>(
            r#"
            SELECT t.id, t.name, t.slug, t.created_by, t.created_at, t.updated_at
            FROM teams t
            JOIN team_members tm ON tm.team_id = t.id
            WHERE tm.user_id = $1
            ORDER BY t.name
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;
        Ok(teams)
    }

    pub(crate) async fn get_by_id(&self, team_id: Uuid) -> Result<Option<Team>> {
        let team = query_as::<_, Team>(
            r#"
            SELECT id, name, slug, created_by, created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
        )
        .bind(team_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(team)
    }

    pub(crate) async fn update(&self, team_id: Uuid, name: &str) -> Result<Option<Team>> {
        let slug = name_to_slug(name);
        let team = query_as::<_, Team>(
            r#"
            UPDATE teams
            SET name = $1, slug = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, slug, created_by, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(&slug)
        .bind(team_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(team)
    }

    pub(crate) async fn delete(&self, team_id: Uuid) -> Result<bool> {
        let rows = query("DELETE FROM teams WHERE id = $1")
            .bind(team_id)
            .execute(&self.db)
            .await?;
        Ok(rows.rows_affected() > 0)
    }

    // ── Membership ──

    pub(crate) async fn get_member_role(
        &self,
        team_id: Uuid,
        user_id: &str,
    ) -> Result<Option<String>> {
        let role: Option<(String,)> =
            query_as("SELECT role FROM team_members WHERE team_id = $1 AND user_id = $2")
                .bind(team_id)
                .bind(user_id)
                .fetch_optional(&self.db)
                .await?;
        Ok(role.map(|r| r.0))
    }

    pub(crate) async fn list_members(&self, team_id: Uuid) -> Result<Vec<TeamMemberWithUser>> {
        let members = query_as::<_, TeamMemberWithUser>(
            r#"
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.created_at,
                   u.email, u.first_name, u.last_name
            FROM team_members tm
            JOIN users u ON u.id = tm.user_id
            WHERE tm.team_id = $1
            ORDER BY tm.created_at
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.db)
        .await?;
        Ok(members)
    }

    pub(crate) async fn add_member(
        &self,
        team_id: Uuid,
        user_id: &str,
        role: &str,
    ) -> Result<Option<TeamMember>> {
        let member = query_as::<_, TeamMember>(
            r#"
            INSERT INTO team_members (team_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (team_id, user_id) DO UPDATE SET role = EXCLUDED.role
            RETURNING id, team_id, user_id, role, created_at
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(role)
        .fetch_optional(&self.db)
        .await?;
        Ok(member)
    }

    pub(crate) async fn update_member_role(
        &self,
        team_id: Uuid,
        user_id: &str,
        role: &str,
    ) -> Result<Option<TeamMember>> {
        let member = query_as::<_, TeamMember>(
            r#"
            UPDATE team_members
            SET role = $1
            WHERE team_id = $2 AND user_id = $3
            RETURNING id, team_id, user_id, role, created_at
            "#,
        )
        .bind(role)
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(member)
    }

    pub(crate) async fn remove_member(&self, team_id: Uuid, user_id: &str) -> Result<bool> {
        let rows = query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(user_id)
            .execute(&self.db)
            .await?;
        Ok(rows.rows_affected() > 0)
    }

    pub(crate) async fn find_user_by_email(&self, email: &str) -> Result<Option<(String,)>> {
        let user = query_as("SELECT id FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.db)
            .await?;
        Ok(user)
    }
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub(crate) struct TeamMemberWithUser {
    pub(crate) id: Uuid,
    pub(crate) team_id: Uuid,
    pub(crate) user_id: String,
    pub(crate) role: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) email: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
}
