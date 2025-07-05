use serde::{ Serialize, Deserialize };
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

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteSymptomRequest {
    pub user_id: String,
    pub logged_at: String, // or chrono::NaiveDate
    pub symptom_type: String,
}

#[derive(Debug, Deserialize)]
pub struct BleedingHistoryRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct BleedingCycle {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: Vec<BleedingDay>,
}

#[derive(Debug, Serialize)]
pub struct BleedingDay {
    pub date: NaiveDate,
    pub intensity: String,
}