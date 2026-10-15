// Точка сборки приложения: здесь настраиваем Tauri и запускаем окно.
// Позже сюда же добавятся команды, которые JS вызывает в Rust.
mod media_keys;
mod power;
mod tray;

use tauri::{AppHandle, Emitter};

/// Метка главного окна (так оно называется в tauri.conf.json по умолчанию).
const MAIN_WINDOW: &str = "main";

/// Отправляет команду плееру в окне ("toggle", "next", "prev", "stop").
/// Сам плеер живёт в JS: там событие `player-action` ловит `__TAURI__.event.listen`.
fn send_player_command(app: &AppHandle, cmd: &str) {
    if let Err(err) = app.emit_to(MAIN_WINDOW, "player-action", cmd) {
        eprintln!("не удалось отправить «{cmd}» в окно: {err}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // команды, которые JS может вызвать через invoke(...)
        .invoke_handler(tauri::generate_handler![power::system_action])
        .setup(|app| {
            tray::create(app.handle())?;
            media_keys::init(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение Tauri");
}
