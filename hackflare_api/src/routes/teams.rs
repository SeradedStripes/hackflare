use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{middlewares::auth_middleware, models::CurrentUser, state::AppState};

// ── Response types ──

#[derive(Serialize)]
struct TeamResponse {
    id: String,
    name: String,
    slug: String,
    role: String,
    member_count: usize,
    created_at: String,
}

#[derive(Serialize)]
struct MemberResponse {
    id: String,
    user_id: String,
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    created_at: String,
}

// ── Request types ──

#[derive(Deserialize)]
struct CreateTeamRequest {
    name: String,
}

#[derive(Deserialize)]
struct UpdateTeamRequest {
    name: String,
}

#[derive(Deserialize)]
struct AddMemberRequest {
    email: String,
    role: Option<String>,
}

#[derive(Deserialize)]
struct UpdateMemberRoleRequest {
    role: String,
}

// ── Helpers ──

fn internal_error(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": msg})),
    )
}

fn not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "not found"})),
    )
}

fn forbidden() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({"error": "forbidden"})),
    )
}

async fn require_team_access(
    state: &AppState,
    team_id: Uuid,
    current_user: &CurrentUser,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state
        .teams
        .get_member_role(team_id, &current_user.user.id)
        .await
    {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(not_found()),
        Err(_) => Err(internal_error("db_error")),
    }
}

async fn require_team_admin(
    state: &AppState,
    team_id: Uuid,
    current_user: &CurrentUser,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state
        .teams
        .get_member_role(team_id, &current_user.user.id)
        .await
    {
        Ok(Some(role)) if role == "owner" || role == "admin" => Ok(()),
        Ok(Some(_)) => Err(forbidden()),
        Ok(None) => Err(not_found()),
        Err(_) => Err(internal_error("db_error")),
    }
}

// ── Team handlers ──

async fn list_teams(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<TeamResponse>>, StatusCode> {
    let teams = state
        .teams
        .list_for_user(&current_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut result = Vec::new();
    for team in teams {
        let members = state
            .teams
            .list_members(team.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let role = state
            .teams
            .get_member_role(team.id, &current_user.user.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .unwrap_or_default();

        result.push(TeamResponse {
            id: team.id.to_string(),
            name: team.name,
            slug: team.slug,
            role,
            member_count: members.len(),
            created_at: team.created_at.to_rfc3339(),
        });
    }

    Ok(Json(result))
}

async fn create_team(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "name is required"})),
        )
            .into_response();
    }

    match state.teams.create(&name, &current_user.user.id).await {
        Ok(team) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": team.id.to_string(),
                "name": team.name,
                "slug": team.slug,
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "failed to create team");
            internal_error("failed to create team").into_response()
        }
    }
}

async fn get_team(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<Uuid>,
) -> impl IntoResponse {
    if require_team_access(&state, team_id, &current_user)
        .await
        .is_err()
    {
        return not_found().into_response();
    }

    let team = match state.teams.get_by_id(team_id).await {
        Ok(Some(t)) => t,
        _ => return not_found().into_response(),
    };

    let members = match state.teams.list_members(team_id).await {
        Ok(m) => m,
        _ => return internal_error("db_error").into_response(),
    };

    let role = match state
        .teams
        .get_member_role(team_id, &current_user.user.id)
        .await
    {
        Ok(Some(r)) => r,
        _ => return internal_error("db_error").into_response(),
    };

    Json(serde_json::json!({
        "id": team.id.to_string(),
        "name": team.name,
        "slug": team.slug,
        "role": role,
        "member_count": members.len(),
        "created_at": team.created_at.to_rfc3339(),
    }))
    .into_response()
}

async fn update_team(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<Uuid>,
    Json(req): Json<UpdateTeamRequest>,
) -> impl IntoResponse {
    if require_team_admin(&state, team_id, &current_user)
        .await
        .is_err()
    {
        return forbidden().into_response();
    }

    let name = req.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "name is required"})),
        )
            .into_response();
    }

    match state.teams.update(team_id, &name).await {
        Ok(Some(team)) => Json(serde_json::json!({
            "id": team.id.to_string(),
            "name": team.name,
            "slug": team.slug,
        }))
        .into_response(),
        Ok(None) => not_found().into_response(),
        Err(_) => internal_error("failed to update team").into_response(),
    }
}

async fn delete_team(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<Uuid>,
) -> impl IntoResponse {
    // Only the owner can delete
    match state
        .teams
        .get_member_role(team_id, &current_user.user.id)
        .await
    {
        Ok(Some(role)) if role == "owner" => {}
        _ => return forbidden().into_response(),
    }

    match state.teams.delete(team_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        _ => not_found().into_response(),
    }
}

// ── Member handlers ──

async fn list_members(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<Uuid>,
) -> impl IntoResponse {
    if require_team_access(&state, team_id, &current_user)
        .await
        .is_err()
    {
        return not_found().into_response();
    }

    match state.teams.list_members(team_id).await {
        Ok(members) => {
            let resp: Vec<MemberResponse> = members
                .into_iter()
                .map(|m| MemberResponse {
                    id: m.id.to_string(),
                    user_id: m.user_id,
                    email: m.email,
                    first_name: m.first_name,
                    last_name: m.last_name,
                    role: m.role,
                    created_at: m.created_at.to_rfc3339(),
                })
                .collect();
            Json(resp).into_response()
        }
        Err(_) => internal_error("db_error").into_response(),
    }
}

async fn add_member(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if require_team_admin(&state, team_id, &current_user)
        .await
        .is_err()
    {
        return forbidden().into_response();
    }

    let email = req.email.trim().to_lowercase();
    if email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "email is required"})),
        )
            .into_response();
    }

    let role = req.role.as_deref().unwrap_or("member");
    if !["member", "admin"].contains(&role) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "role must be 'member' or 'admin'"})),
        )
            .into_response();
    }

    let user = match state.teams.find_user_by_email(&email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "user not found"})),
            )
                .into_response();
        }
        Err(_) => return internal_error("db_error").into_response(),
    };

    match state.teams.add_member(team_id, &user.0, role).await {
        Ok(Some(member)) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": member.id.to_string(),
                "user_id": member.user_id,
                "role": member.role,
            })),
        )
            .into_response(),
        Ok(None) => internal_error("failed to add member").into_response(),
        Err(_) => internal_error("db_error").into_response(),
    }
}

async fn update_member_role_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((team_id, user_id)): Path<(Uuid, String)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> impl IntoResponse {
    // Only owner/admin can change roles; cannot change owner's role
    match state
        .teams
        .get_member_role(team_id, &current_user.user.id)
        .await
    {
        Ok(Some(role)) if role == "owner" || role == "admin" => {}
        _ => return forbidden().into_response(),
    }

    let new_role = req.role.trim().to_string();
    if !["member", "admin"].contains(&new_role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "role must be 'member' or 'admin'"})),
        )
            .into_response();
    }

    let target_role = match state.teams.get_member_role(team_id, &user_id).await {
        Ok(Some(r)) => r,
        _ => return not_found().into_response(),
    };

    // Cannot change the owner's role
    if target_role == "owner" {
        return forbidden().into_response();
    }

    // Only owner can promote to admin
    let current_role = state
        .teams
        .get_member_role(team_id, &current_user.user.id)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if new_role == "admin" && current_role != "owner" {
        return forbidden().into_response();
    }

    match state
        .teams
        .update_member_role(team_id, &user_id, &new_role)
        .await
    {
        Ok(Some(member)) => Json(serde_json::json!({
            "id": member.id.to_string(),
            "user_id": member.user_id,
            "role": member.role,
        }))
        .into_response(),
        _ => internal_error("db_error").into_response(),
    }
}

async fn remove_member(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((team_id, user_id)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    // Allow self-removal (leave team) or owner/admin removal
    let is_self = user_id == current_user.user.id;

    if !is_self {
        match state
            .teams
            .get_member_role(team_id, &current_user.user.id)
            .await
        {
            Ok(Some(role)) if role == "owner" || role == "admin" => {}
            _ => return forbidden().into_response(),
        }
    }

    let target_role = match state.teams.get_member_role(team_id, &user_id).await {
        Ok(Some(r)) => r,
        _ => return not_found().into_response(),
    };

    // Cannot remove the owner
    if target_role == "owner" {
        return forbidden().into_response();
    }

    // Admin cannot remove other admins (only owner can)
    if !is_self && target_role == "admin" {
        let current_role = state
            .teams
            .get_member_role(team_id, &current_user.user.id)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        if current_role != "owner" {
            return forbidden().into_response();
        }
    }

    match state.teams.remove_member(team_id, &user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        _ => not_found().into_response(),
    }
}

// ── Router ──

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_teams).post(create_team))
        .route(
            "/{team_id}",
            get(get_team).put(update_team).delete(delete_team),
        )
        .route("/{team_id}/members", get(list_members).post(add_member))
        .route(
            "/{team_id}/members/{user_id}",
            put(update_member_role_handler).delete(remove_member),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
