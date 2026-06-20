use chrono::{Datelike, Local, Timelike};
use tauri::{AppHandle, Manager};

use crate::{db, AppState};

/// true si la plataforma debe estar bloqueada según sus horarios individuales.
/// Los schedules representan ventanas PERMITIDAS; sin schedules = bloqueado 24/7.
pub fn should_block(schedules: &[db::Schedule]) -> bool {
    if schedules.is_empty() {
        return true;
    }

    let now = Local::now();
    let current_day = now.weekday().num_days_from_monday() as i32;
    let current_time = format!("{:02}:{:02}", now.hour(), now.minute());

    for s in schedules {
        let day_match = s.day_of_week == 7 || s.day_of_week == current_day;
        if day_match && current_time >= s.start_time && current_time < s.end_time {
            return false;
        }
    }

    true
}

/// true si algún horario global cubre el momento actual.
/// Los horarios globales representan ventanas BLOQUEADAS.
pub fn is_globally_blocked(global_schedules: &[db::GlobalSchedule]) -> bool {
    let now = Local::now();
    let current_day = now.weekday().num_days_from_monday() as i32;
    let current_time = format!("{:02}:{:02}", now.hour(), now.minute());

    for gs in global_schedules {
        if gs.days.contains(&current_day)
            && current_time >= gs.start_time
            && current_time < gs.end_time
        {
            return true;
        }
    }

    false
}

pub fn run_scheduler(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
        let state = app.state::<AppState>();
        if let Err(e) = crate::refresh_blocks(&state) {
            eprintln!("[FocusGuard scheduler] Error: {}", e);
        }
    });
}
