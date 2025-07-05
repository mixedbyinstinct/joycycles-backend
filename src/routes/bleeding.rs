use axum::{extract::{State, Query}, Json, Router, routing::get};
use sqlx::PgPool;
use chrono::{NaiveDate, Duration};
use std::collections::BTreeMap;
use crate::models::{BleedingCycle, BleedingDay, BleedingHistoryRequest};

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/bleeding-history", get(get_bleeding_history))
        .with_state(pool)
}

pub async fn get_bleeding_history(
    State(pool): State<PgPool>,
    Query(params): Query<BleedingHistoryRequest>,
) -> Result<Json<Vec<BleedingCycle>>, (axum::http::StatusCode, String)> {
    let rows = sqlx::query!(
        "SELECT logged_at::date as date, intensity
         FROM symptom_logs
         WHERE user_id = $1 AND symptom_type = 'bleeding'
         ORDER BY logged_at ASC",
        params.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ DB error: {:?}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "DB error".into())
    })?;

    let mut grouped: Vec<BleedingCycle> = vec![];
    let mut current_cycle: Vec<BleedingDay> = vec![];

    for (i, row) in rows.iter().enumerate() {
        let date = row.date;
        let intensity = row.intensity.clone().unwrap_or_default();
    
        let is_start = i == 0;
        let prev_date = rows.get(i.wrapping_sub(1)).map(|r| r.date);
        let gap = prev_date.map(|d| date.signed_duration_since(d).num_days());
    
        if is_start || gap == Some(1) {
            current_cycle.push(BleedingDay { date, intensity });
        } else {
            if !current_cycle.is_empty() {
                let first = current_cycle.first().unwrap().date;
                let last = current_cycle.last().unwrap().date;
                grouped.push(BleedingCycle {
                    start_date: first,
                    end_date: last,
                    days: current_cycle.drain(..).collect(),
                });
            }
            current_cycle.push(BleedingDay { date, intensity });
        }
    }
    

    if !current_cycle.is_empty() {
        let first = current_cycle.first().unwrap().date;
        let last = current_cycle.last().unwrap().date;
        grouped.push(BleedingCycle {
            start_date: first,
            end_date: last,
            days: current_cycle,
        });
    }

    Ok(Json(grouped))
}
