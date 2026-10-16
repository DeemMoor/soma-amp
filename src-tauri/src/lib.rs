// Точка сборки приложения: здесь настраиваем Tauri и запускаем окно.
// Позже сюда же добавятся команды, которые JS вызывает в Rust.
#[macro_use]
mod logfile;
mod autostart;
mod links;
mod media_keys;
mod power;
mod settings;
mod tray;
mod updates;

use tauri::{AppHandle, Emitter, Manager};

/// Метка главного окна (так оно называется в tauri.conf.json по умолчанию).
const MAIN_WINDOW: &str = "main";

/// Отправляет команду плееру в окне ("toggle", "next", "prev", "stop").
/// Сам плеер живёт в JS: там событие `player-action` ловит `__TAURI__.event.listen`.
fn send_player_command(app: &AppHandle, cmd: &str) {
    if let Err(err) = app.emit_to(MAIN_WINDOW, "player-action", cmd) {
        log_error!("не удалось отправить «{cmd}» в окно: {err}");
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
        log_error!("не удалось показать окно: {err}");
    }
}

/// Адрес встроенного сервера в установленной программе — как у `cargo tauri dev`.
/// Именно IP 127.0.0.1, а не имя localhost: SomaFM отдаёт поток странице с http://127.0.0.1:…,
/// а с http://localhost:… и http://tauri.localhost отвечает 403 (проверено на этапе 6.1).
const LOCALHOST_HOST: &str = "127.0.0.1";
/// Порты, которые пробуем по очереди: если 1430 занят другой программой, берём следующий.
#[cfg_attr(debug_assertions, allow(dead_code))]
const LOCALHOST_PORTS: std::ops::RangeInclusive<u16> = 1430..=1439;

/// Первый свободный порт из диапазона. Проверяем, заняв его на миг и сразу отпустив.
#[cfg_attr(debug_assertions, allow(dead_code))]
fn find_free_port(host: &str, ports: std::ops::RangeInclusive<u16>) -> Option<u16> {
    ports
        .into_iter()
        .find(|&port| std::net::TcpListener::bind((host, port)).is_ok())
}

/// Создаёт главное окно по описанию из tauri.conf.json (там у него `create: false`).
/// Создаём сами, а не отдаём Tauri, чтобы ещё до запуска страницы положить в окно настройки.
/// `server_port` — порт нашего встроенного сервера; `None` — страница из самой программы
/// (в `cargo tauri dev` это сервер Tauri CLI).
fn create_main_window(
    app: &mut tauri::App,
    server_port: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = settings::SettingsStore::open(app.handle())?;
    let script = store.init_script()?;
    app.manage(store);

    let mut config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == MAIN_WINDOW)
        .ok_or("в tauri.conf.json нет окна main")?
        .clone();
    if let Some(port) = server_port {
        let url = format!("http://{LOCALHOST_HOST}:{port}");
        config.url = tauri::WebviewUrl::External(url.parse()?);
    }
    tauri::WebviewWindowBuilder::from_config(app.handle(), &config)?
        .initialization_script(&script)
        .build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // встроенный сервер — только в установленной программе: в cargo tauri dev его роль
    // играет сервер Tauri CLI на том же 127.0.0.1:1430
    #[cfg(not(debug_assertions))]
    let server_port = find_free_port(LOCALHOST_HOST, LOCALHOST_PORTS);
    #[cfg(debug_assertions)]
    let server_port: Option<u16> = None;

    let mut builder = tauri::Builder::default()
        // один экземпляр: второй запуск не открывает новое окно, а будит первое.
        // Плагин подключаем первым — чтобы второй экземпляр закрылся раньше, чем что-то создаст
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }));
    if let Some(port) = server_port {
        builder = builder.plugin(
            tauri_plugin_localhost::Builder::new(port)
                .host(LOCALHOST_HOST)
                .build(),
        );
    }
    builder
        // автообновление с GitHub Releases (адрес и публичный ключ — в tauri.conf.json)
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            links::open_url,
            updates::check_update,
            updates::install_update,
            logfile::write_log,
        ])
        .setup(move |app| {
            logfile::init(app.path().app_log_dir()?);
            log_info!("запуск Soma Amp {}", app.package_info().version);
            match server_port {
                Some(port) => log_info!("встроенный сервер: {LOCALHOST_HOST}:{port}"),
                None if !cfg!(debug_assertions) => log_error!(
                    "порты {}–{} заняты: плеер открыт без встроенного сервера, звук SomaFM работать не будет",
                    LOCALHOST_PORTS.start(),
                    LOCALHOST_PORTS.end()
                ),
                None => {}
            }
            create_main_window(app, server_port)?;
            tray::create(app.handle())?;
            media_keys::init(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_busy_port() {
        // занимаем первый порт диапазона — функция должна взять следующий свободный
        let busy = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = busy.local_addr().unwrap().port();
        let found = find_free_port("127.0.0.1", port..=port.saturating_add(20));
        assert!(found.is_some_and(|p| p != port));
    }

    #[test]
    fn all_busy_means_none() {
        let busy = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = busy.local_addr().unwrap().port();
        assert_eq!(find_free_port("127.0.0.1", port..=port), None);
    }
}
