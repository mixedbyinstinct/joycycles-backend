use axum::{
    Router,
    routing::{get, post},
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::BTreeMap;
use crate::models::{ SymptomsByDate, SymptomLog };

#[derive(Deserialize)]
pub struct NewSymptom {
    pub user_id: Uuid,
    pub logged_at: NaiveDate,
    pub symptom_type: String,
    pub intensity: Option<String>,
}

#[derive(Deserialize)]
struct UserQuery {
    user_id: Uuid,
}


pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/symptom", post(log_symptom))
        .route("/symptoms", get(get_symptoms_grouped))
        .route("/symptom/all", get(get_symptoms_flat))
        .with_state(pool)
}

async fn get_symptoms_flat(
    State(pool): State<PgPool>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Vec<SymptomLog>>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT logged_at, symptom_type, intensity
        FROM symptom_logs
        WHERE user_id = $1
        ORDER BY logged_at DESC
        "#,
        query.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch flat symptoms: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let logs: Vec<SymptomLog> = rows
        .into_iter()
        .map(|row| SymptomLog {
            logged_at: row.logged_at,
            symptom_type: row.symptom_type,
            intensity: row.intensity,
        })
        .collect();

    Ok(Json(logs))
}

async fn log_symptom(
    State(pool): State<PgPool>,
    Json(body): Json<NewSymptom>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query!(
        "INSERT INTO symptom_logs (user_id, logged_at, symptom_type, intensity) VALUES ($1, $2, $3, $4)",
        body.user_id,
        body.logged_at,
        body.symptom_type,
        body.intensity
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

async fn get_symptoms_grouped(
    State(pool): State<PgPool>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Vec<SymptomsByDate>>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT logged_at, symptom_type
        FROM symptom_logs
        WHERE user_id = $1
        ORDER BY logged_at DESC
        "#,
        query.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("❌ Failed to fetch symptoms: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut map = BTreeMap::<NaiveDate, Vec<String>>::new();
    for row in rows {
        map.entry(row.logged_at)
            .or_default()
            .push(row.symptom_type);
    }

    let result: Vec<SymptomsByDate> = map
        .into_iter()
        .map(|(logged_at, symptoms)| SymptomsByDate { logged_at, symptoms })
        .collect();

    Ok(Json(result))
}
