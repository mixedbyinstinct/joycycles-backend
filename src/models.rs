use serde::Serialize;
use uuid::Uuid;
use chrono::{NaiveDate, DateTime, Utc};

#[derive(Serialize)]
pub struct Cycle {
    pub id: Uuid,
    pub user_id: Uuid,
    pub start_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct CycleSummary {
    pub cycle_day: i64,
    pub in_fertile_window: bool,
    pub period_expected_in_days: i64,
    pub start_date: NaiveDate,
}

#[derive(Serialize)]
pub struct SymptomsByDate {
    pub logged_at: NaiveDate,
    pub symptoms: Vec<String>,
}

#[derive(Serialize)]
pub struct SymptomLog {
    pub logged_at: NaiveDate,
    pub symptom_type: String,
    pub intensity: Option<String>,
}
