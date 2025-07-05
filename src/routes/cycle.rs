use axum::{Router, routing::{get, post}, Json, extract::{State, Query}};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use crate::models::{Cycle, CycleSummary};
use axum::http::StatusCode;

#[derive(Deserialize)]
pub struct UserQuery {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct NewCycle {
    pub user_id: Uuid,
    pub start_date: NaiveDate,
}

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/cycle", get(get_cycle_summary).post(create_cycle))
        .with_state(pool)
}

async fn create_cycle(
    State(pool): State<PgPool>,
    Json(body): Json<NewCycle>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query!(
        "INSERT INTO cycles (user_id, start_date) VALUES ($1, $2)",
        body.user_id,
        body.start_date
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            tracing::error!("❌ DB insert failed: {}", db_err.message());
    
            if let Some(code) = db_err.code() {
                tracing::info!("ℹ️ SQLSTATE code: {}", code);
            }
    
            if let Some(constraint) = db_err.constraint() {
                tracing::info!("🔒 Constraint violated: {}", constraint);
            }
        } else {
            tracing::error!("❌ Unknown DB error: {}", e);
        }
    
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    
    Ok(StatusCode::CREATED)
}

async fn get_cycle_summary(
    State(pool): State<PgPool>,
    Query(params): Query<UserQuery>,
) -> Result<Json<CycleSummary>, StatusCode> {
    let Some(cycle) = sqlx::query_as!(
        Cycle,
        "SELECT id, user_id, start_date, created_at FROM cycles WHERE user_id = $1 ORDER BY start_date DESC LIMIT 1",
        params.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        eprintln!("❌ DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })? else {
        return Err(StatusCode::NOT_FOUND);
    };

    let today = chrono::Utc::now().naive_utc().date();
    let cycle_day = (today - cycle.start_date).num_days();

    let fertile = cycle.start_date + chrono::Duration::days(12)..=cycle.start_date + chrono::Duration::days(16);
    let period_expected_in_days = 26 - cycle_day;

    Ok(Json(CycleSummary {
        cycle_day,
        in_fertile_window: fertile.contains(&today),
        period_expected_in_days,
        start_date: cycle.start_date,
    }))
}
