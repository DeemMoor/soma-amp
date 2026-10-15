// Иконка в трее (область уведомлений Windows) и её меню.
// Плеер живёт в JS, поэтому команды плеера Rust не выполняет сам,
// а отправляет окну событие `tray-action`, и уже JS вызывает play()/pause()/step().

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

/// Метка главного окна (так оно называется в tauri.conf.json по умолчанию).
const MAIN_WINDOW: &str = "main";

/// Что делать по пункту меню трея.
#[derive(Debug, PartialEq)]
enum TrayAction {
    /// Команда плееру — передаём её в JS как есть: "toggle", "next", "prev".
    Player(&'static str),
    /// Показать окно плеера.
    Show,
    /// Закрыть приложение совсем.
    Quit,
}

/// Переводит id пункта меню в действие. Неизвестный id — `None`.
fn tray_action(id: &str) -> Option<TrayAction> {
    match id {
        "toggle" => Some(TrayAction::Player("toggle")),
        "next" => Some(TrayAction::Player("next")),
        "prev" => Some(TrayAction::Player("prev")),
        "show" => Some(TrayAction::Show),
        "quit" => Some(TrayAction::Quit),
        _ => None,
    }
}

/// Создаёт иконку в трее с меню. Вызывается один раз при запуске.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Играть / Пауза", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "Следующий канал", true, None::<&str>)?;
    let prev = MenuItem::with_id(app, "prev", "Предыдущий канал", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Показать", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&toggle, &next, &prev, &sep1, &show, &sep2, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Soma Amp")
        .menu(&menu)
        // левый клик показывает окно, меню — по правому, как у всех программ в трее
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    // иконку берём ту же, что у окна (из icons/)
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn handle_menu(app: &AppHandle, id: &str) {
    match tray_action(id) {
        Some(TrayAction::Player(cmd)) => {
            if let Err(err) = app.emit_to(MAIN_WINDOW, "tray-action", cmd) {
                eprintln!("трей: не удалось отправить «{cmd}» в окно: {err}");
            }
        }
        Some(TrayAction::Show) => show_main_window(app),
        Some(TrayAction::Quit) => app.exit(0),
        None => {}
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
        eprintln!("трей: не удалось показать окно: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_commands_go_to_js() {
        assert_eq!(tray_action("toggle"), Some(TrayAction::Player("toggle")));
        assert_eq!(tray_action("next"), Some(TrayAction::Player("next")));
        assert_eq!(tray_action("prev"), Some(TrayAction::Player("prev")));
    }

    #[test]
    fn window_commands() {
        assert_eq!(tray_action("show"), Some(TrayAction::Show));
        assert_eq!(tray_action("quit"), Some(TrayAction::Quit));
    }

    #[test]
    fn unknown_id_is_ignored() {
        assert_eq!(tray_action("reboot"), None);
        assert_eq!(tray_action(""), None);
    }
}
