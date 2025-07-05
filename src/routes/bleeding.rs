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
    let mut rows = sqlx::query!(
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

    // Sort rows just in case (paranoia — even though the SQL query already orders them)
    rows.sort_by_key(|r| r.date);

    let mut grouped: Vec<BleedingCycle> = vec![];
    let mut current_cycle: Vec<BleedingDay> = vec![];

    let mut prev_date_opt: Option<NaiveDate> = None;

    for row in rows {
        let date = row.date;
        let intensity = row.intensity.unwrap_or_else(|| "".to_string());

        match prev_date_opt {
            Some(prev_date) if (date - prev_date).num_days() > 1 => {
                // gap detected — finalize current cycle
                if !current_cycle.is_empty() {
                    grouped.push(BleedingCycle {
                        start_date: current_cycle.first().unwrap().date,
                        end_date: current_cycle.last().unwrap().date,
                        days: current_cycle.drain(..).collect(),
                    });
                }
            }
            _ => {} // no gap, continue adding to current cycle
        }

        current_cycle.push(BleedingDay { date, intensity });
        prev_date_opt = Some(date);
    }

    // Push final cycle
    if !current_cycle.is_empty() {
        grouped.push(BleedingCycle {
            start_date: current_cycle.first().unwrap().date,
            end_date: current_cycle.last().unwrap().date,
            days: current_cycle,
        });
    }

    Ok(Json(grouped))
}

