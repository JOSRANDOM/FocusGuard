use std::collections::HashSet;
use std::sync::Mutex;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, Wry,
};
use tauri_plugin_notification::NotificationExt;

mod blocker;
mod db;
mod scheduler;

pub use db::{GlobalSchedule, NewGlobalSchedule, NewSchedule, Platform, Schedule};

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    /// IDs de plataformas bloqueadas en el último tick del scheduler, para
    /// notificar solo las que acaban de pasar de desbloqueada a bloqueada.
    pub previously_blocked: Mutex<HashSet<i64>>,
    /// Ícono de bandeja: solo se muestra mientras algo esté bloqueado.
    pub tray: Mutex<Option<TrayIcon<Wry>>>,
}

// ─── Plataformas ──────────────────────────────────────────────────────────────

#[tauri::command]
fn get_platforms(state: State<AppState>) -> Result<Vec<Platform>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut platforms = db::get_platforms(&conn).map_err(|e| e.to_string())?;
    let global = db::get_global_schedules(&conn).map_err(|e| e.to_string())?;
    let globally_blocked = scheduler::is_globally_blocked(&global);

    for p in &mut platforms {
        if p.enabled {
            if globally_blocked {
                p.currently_blocked = true;
            } else {
                let schedules = db::get_schedules(&conn, p.id).map_err(|e| e.to_string())?;
                p.currently_blocked = scheduler::should_block(&schedules);
            }
        }
    }

    Ok(platforms)
}

#[tauri::command]
fn toggle_platform(id: i64, enabled: bool, state: State<AppState>) -> Result<(), String> {
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::toggle_platform(&conn, id, enabled).map_err(|e| e.to_string())?;
    }
    refresh_blocks(&state)
}

#[tauri::command]
fn get_schedules(platform_id: i64, state: State<AppState>) -> Result<Vec<Schedule>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_schedules(&conn, platform_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_schedule(schedule: NewSchedule, state: State<AppState>) -> Result<Schedule, String> {
    let id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::add_schedule(&conn, &schedule).map_err(|e| e.to_string())?
    };
    refresh_blocks(&state)?;
    Ok(Schedule {
        id,
        platform_id: schedule.platform_id,
        day_of_week: schedule.day_of_week,
        start_time: schedule.start_time,
        end_time: schedule.end_time,
    })
}

#[tauri::command]
fn delete_schedule(id: i64, state: State<AppState>) -> Result<(), String> {
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::delete_schedule(&conn, id).map_err(|e| e.to_string())?;
    }
    refresh_blocks(&state)
}

// ─── Horarios Globales ────────────────────────────────────────────────────────

#[tauri::command]
fn get_global_schedules(state: State<AppState>) -> Result<Vec<GlobalSchedule>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_global_schedules(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_global_schedule(
    schedule: NewGlobalSchedule,
    state: State<AppState>,
) -> Result<GlobalSchedule, String> {
    if schedule.days.is_empty() {
        return Err("Debes seleccionar al menos un día".to_string());
    }
    if schedule.start_time >= schedule.end_time {
        return Err("La hora de inicio debe ser anterior a la de fin".to_string());
    }
    let id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::add_global_schedule(&conn, &schedule).map_err(|e| e.to_string())?
    };
    refresh_blocks(&state)?;
    Ok(GlobalSchedule {
        id,
        days: schedule.days,
        start_time: schedule.start_time,
        end_time: schedule.end_time,
        platforms: schedule.platforms,
    })
}

#[tauri::command]
fn delete_global_schedule(id: i64, state: State<AppState>) -> Result<(), String> {
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::delete_global_schedule(&conn, id).map_err(|e| e.to_string())?;
    }
    refresh_blocks(&state)
}

// ─── Aplicar bloqueos ─────────────────────────────────────────────────────────

#[tauri::command]
fn apply_blocks_now(state: State<AppState>) -> Result<Vec<String>, String> {
    refresh_blocks(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let platforms = db::get_platforms(&conn).map_err(|e| e.to_string())?;
    let global = db::get_global_schedules(&conn).map_err(|e| e.to_string())?;
    let globally_blocked = scheduler::is_globally_blocked(&global);

    let mut blocked = Vec::new();
    for p in &platforms {
        if p.enabled {
            let is_blocked = if globally_blocked {
                true
            } else {
                let schedules = db::get_schedules(&conn, p.id).map_err(|e| e.to_string())?;
                scheduler::should_block(&schedules)
            };
            if is_blocked {
                blocked.push(p.name.clone());
            }
        }
    }
    Ok(blocked)
}

/// Compara qué plataformas están bloqueadas ahora contra el último tick del
/// scheduler y envía una notificación nativa solo por las que acaban de pasar
/// de desbloqueada a bloqueada (evita notificar de nuevo cada 60s mientras
/// una plataforma sigue bloqueada).
pub fn notify_new_blocks(app: &AppHandle, state: &State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let platforms = db::get_platforms(&conn).map_err(|e| e.to_string())?;
    let global = db::get_global_schedules(&conn).map_err(|e| e.to_string())?;
    let globally_blocked = scheduler::is_globally_blocked(&global);

    let mut currently_blocked_ids = HashSet::new();

    for p in &platforms {
        if !p.enabled {
            continue;
        }
        let is_blocked = if globally_blocked {
            true
        } else {
            let schedules = db::get_schedules(&conn, p.id).map_err(|e| e.to_string())?;
            scheduler::should_block(&schedules)
        };
        if is_blocked {
            currently_blocked_ids.insert(p.id);
        }
    }
    drop(conn);

    let mut prev = state.previously_blocked.lock().map_err(|e| e.to_string())?;
    let newly_blocked: Vec<String> = platforms
        .iter()
        .filter(|p| currently_blocked_ids.contains(&p.id) && !prev.contains(&p.id))
        .map(|p| p.name.clone())
        .collect();
    *prev = currently_blocked_ids;
    drop(prev);

    if !newly_blocked.is_empty() {
        let body = if newly_blocked.len() == 1 {
            format!("{} está bloqueado ahora.", newly_blocked[0])
        } else {
            format!("{} están bloqueados ahora.", newly_blocked.join(", "))
        };
        let _ = app
            .notification()
            .builder()
            .title("🛡️ FocusGuard")
            .body(body)
            .show();
    }

    Ok(())
}

pub fn refresh_blocks(state: &State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let platforms = db::get_platforms(&conn).map_err(|e| e.to_string())?;
    let global_schedules = db::get_global_schedules(&conn).map_err(|e| e.to_string())?;

    let mut domains_to_block: Vec<String> = Vec::new();

    for p in &platforms {
        if !p.enabled {
            continue;
        }

        // ¿Algún horario global cubre esta plataforma ahora?
        let blocked_by_global = global_schedules.iter().any(|gs| {
            let covers_platform = gs.platforms.is_empty() || gs.platforms.contains(&p.id);
            covers_platform && scheduler::is_globally_blocked(std::slice::from_ref(gs))
        });

        let is_blocked = if blocked_by_global {
            true
        } else {
            let schedules = db::get_schedules(&conn, p.id).map_err(|e| e.to_string())?;
            scheduler::should_block(&schedules)
        };

        if is_blocked {
            domains_to_block.extend(p.domains.clone());
        }
    }

    drop(conn);

    if let Ok(tray) = state.tray.lock() {
        if let Some(tray) = tray.as_ref() {
            let _ = tray.set_visible(!domains_to_block.is_empty());
        }
    }

    blocker::apply_blocks(&domains_to_block)
}

// ─── Setup helper (macOS) ─────────────────────────────────────────────────────

#[tauri::command]
fn check_helper_installed() -> bool {
    blocker::is_helper_installed()
}

#[tauri::command]
fn install_helper() -> Result<(), String> {
    blocker::install_helper()
}

// ─── Private Relay (macOS) ────────────────────────────────────────────────────

#[tauri::command]
fn check_private_relay() -> bool {
    #[cfg(target_os = "macos")]
    {
        // cloudrelayproxyd corre cuando iCloud Private Relay está activo
        std::process::Command::new("pgrep")
            .args(["-x", "cloudrelayproxyd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = db::init_db().expect("Error iniciando la base de datos");
    let app_state = AppState {
        db: Mutex::new(conn),
        previously_blocked: Mutex::new(HashSet::new()),
        tray: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_platforms,
            toggle_platform,
            get_schedules,
            add_schedule,
            delete_schedule,
            get_global_schedules,
            add_global_schedule,
            delete_global_schedule,
            apply_blocks_now,
            check_private_relay,
            check_helper_installed,
            install_helper,
        ])
        .setup(|app| {
            // En Windows esto es prácticamente un no-op; en macOS dispara el
            // diálogo de permiso de notificaciones la primera vez que corre
            // la app, para que las notificaciones de bloqueo funcionen sin
            // tocar nada desde el frontend.
            let _ = app.notification().request_permission();

            if let Some(icon) = app.default_window_icon().cloned() {
                let tray = TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("FocusGuard está activo")
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;

                // Arranca oculto: solo debe verse mientras haya algo bloqueado
                // de verdad. refresh_blocks() lo muestra/oculta según toque.
                let _ = tray.set_visible(false);
                *app.state::<AppState>().tray.lock().unwrap() = Some(tray);
            }

            let handle = app.handle().clone();
            scheduler::run_scheduler(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error iniciando FocusGuard");
}
