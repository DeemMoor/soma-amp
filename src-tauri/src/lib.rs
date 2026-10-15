// Точка сборки приложения: здесь настраиваем Tauri и запускаем окно.
// Позже сюда же добавятся команды, которые JS вызывает в Rust.
mod autostart;
mod media_keys;
mod power;
mod settings;
mod tray;

use tauri::{AppHandle, Emitter, Manager};

/// Метка главного окна (так оно называется в tauri.conf.json по умолчанию).
const MAIN_WINDOW: &str = "main";

/// Отправляет команду плееру в окне ("toggle", "next", "prev", "stop").
/// Сам плеер живёт в JS: там событие `player-action` ловит `__TAURI__.event.listen`.
fn send_player_command(app: &AppHandle, cmd: &str) {
    if let Err(err) = app.emit_to(MAIN_WINDOW, "player-action", cmd) {
        eprintln!("не удалось отправить «{cmd}» в окно: {err}");
    }
}

/// Показывает окно плеера: из трея, из свёрнутого состояния, и делает его активным.
fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let result = window
        .show()
        .and_then(|()| window.unminimize())
        .and_then(|()| window.set_focus());
    if let Err(err) = result {
        eprintln!("не удалось показать окно: {err}");
    }
}

/// Создаёт главное окно по описанию из tauri.conf.json (там у него `create: false`).
/// Создаём сами, а не отдаём Tauri, чтобы ещё до запуска страницы положить в окно настройки.
fn create_main_window(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let store = settings::SettingsStore::open(app.handle())?;
    let script = store.init_script()?;
    app.manage(store);

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == MAIN_WINDOW)
        .ok_or("в tauri.conf.json нет окна main")?;
    tauri::WebviewWindowBuilder::from_config(app.handle(), config)?
        .initialization_script(&script)
        .build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // один экземпляр: второй запуск не открывает новое окно, а будит первое.
        // Плагин подключаем первым — чтобы второй экземпляр закрылся раньше, чем что-то создаст
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        // автозапуск с Windows; LaunchAgent — способ для macOS, на Windows не используется
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // команды, которые JS может вызвать через invoke(...)
        .invoke_handler(tauri::generate_handler![
            power::system_action,
            autostart::autostart_enabled,
            autostart::set_autostart,
            settings::settings_set,
            settings::settings_import,
        ])
        .setup(|app| {
            create_main_window(app)?;
            tray::create(app.handle())?;
            media_keys::init(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение Tauri");
}
