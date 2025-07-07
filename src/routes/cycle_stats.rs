use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CycleStatsQuery {
    user_id: Uuid,
}

#[derive(Serialize)]
pub struct CycleStat {
    cycle_number: i32,
    period_length: i32,
    cycle_length: i32,
}

#[derive(Serialize)]
pub struct CycleStatsResponse {
    average_period_length: f64,
    average_cycle_length: f64,
    cycle_stats: Vec<CycleStat>,
}

pub async fn get_cycle_stats(
    State(pool): State<PgPool>,
    Query(query): Query<CycleStatsQuery>,
) -> Result<Json<CycleStatsResponse>, StatusCode> {
    let user_id = query.user_id;

    let rows = sqlx::query!(
        r#"
        WITH ordered_cycles AS (
            SELECT
                id,
                start_date,
                LEAD(start_date) OVER (ORDER BY start_date) AS next_start_date
            FROM cycles
            WHERE user_id = $1
        )
        SELECT
            c.start_date,
            c.next_start_date,
            (
                SELECT COUNT(*) FROM symptom_logs s
                WHERE s.user_id = $1
                  AND s.symptom_type = 'bleeding'
                  AND s.logged_at >= c.start_date
                  AND (
                        c.next_start_date IS NULL OR s.logged_at < c.next_start_date
                  )
            ) AS period_length
        FROM ordered_cycles c
        ORDER BY c.start_date ASC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ DB error in get_cycle_stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut total_period = 0;
    let mut total_cycle = 0;
    let mut stats = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        let period = row.period_length.unwrap_or(0);
        let cycle_len = row.next_start_date
            .map(|next| (next - row.start_date).num_days())
            .unwrap_or(0); // 0 for ongoing cycle

        total_period += period;
        total_cycle += cycle_len;

        stats.push(CycleStat {
            cycle_number: (i + 1) as i32,
            period_length: period as i32,
            cycle_length: cycle_len as i32,
        });
    }

    let count = stats.len() as f64;

    Ok(Json(CycleStatsResponse {
        average_period_length: if count > 0.0 { total_period as f64 / count } else { 0.0 },
        average_cycle_length: if count > 0.0 { total_cycle as f64 / count } else { 0.0 },
        cycle_stats: stats,
    }))
}

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/cycle-stats", get(get_cycle_stats))
        .with_state(pool)
}
