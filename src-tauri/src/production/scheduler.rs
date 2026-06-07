use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::models::CountdownSchedule;

pub fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Returns true when the schedule should fire (countdown should auto-start).
pub fn should_fire(schedule: &CountdownSchedule) -> bool {
    if !schedule.enabled || schedule.fired {
        return false;
    }
    let now = unix_now_secs();
    let start_at = schedule
        .service_at_unix
        .saturating_sub(schedule.lead_secs as u64);
    now >= start_at
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStatus {
    pub schedule: CountdownSchedule,
    pub seconds_until_start: i64,
    pub ready: bool,
}

pub fn schedule_status(schedule: &CountdownSchedule) -> ScheduleStatus {
    let now = unix_now_secs() as i64;
    let start_at = schedule.service_at_unix.saturating_sub(schedule.lead_secs as u64) as i64;
    ScheduleStatus {
        schedule: schedule.clone(),
        seconds_until_start: start_at - now,
        ready: should_fire(schedule),
    }
}
