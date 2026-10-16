// Глобальные медиаклавиши клавиатуры: Play/Pause, Next, Prev, Stop.
// Регистрируем их в Windows как горячие клавиши, поэтому они работают,
// даже когда окно плеера свёрнуто или спрятано в трей.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

/// Медиаклавиши, которые слушаем.
const KEYS: [Code; 4] = [
    Code::MediaPlayPause,
    Code::MediaTrackNext,
    Code::MediaTrackPrevious,
    Code::MediaStop,
];

/// Какую команду плееру даёт клавиша. Не медиаклавиша — `None`.
fn player_command(key: Code) -> Option<&'static str> {
    match key {
        Code::MediaPlayPause => Some("toggle"),
        Code::MediaTrackNext => Some("next"),
        Code::MediaTrackPrevious => Some("prev"),
        Code::MediaStop => Some("stop"),
        _ => None,
    }
}

/// Подключает плагин горячих клавиш и регистрирует медиаклавиши.
pub fn init(app: &AppHandle) -> tauri::Result<()> {
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, event| {
                // реагируем на нажатие, а не на отпускание — иначе команда пришла бы дважды
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                if let Some(cmd) = player_command(shortcut.key) {
                    crate::send_player_command(app, cmd);
                }
            })
            .build(),
    )?;

    for key in KEYS {
        // клавишу мог уже занять другой плеер — это не повод падать, просто предупреждаем
        if let Err(err) = app.global_shortcut().register(Shortcut::new(None, key)) {
            crate::log_error!("медиаклавиши: не удалось занять {key:?}: {err}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_key_has_a_command() {
        for key in KEYS {
            assert!(player_command(key).is_some(), "{key:?} без команды");
        }
    }

    #[test]
    fn media_keys_map_to_player_commands() {
        assert_eq!(player_command(Code::MediaPlayPause), Some("toggle"));
        assert_eq!(player_command(Code::MediaTrackNext), Some("next"));
        assert_eq!(player_command(Code::MediaTrackPrevious), Some("prev"));
        assert_eq!(player_command(Code::MediaStop), Some("stop"));
    }

    #[test]
    fn other_keys_are_ignored() {
        assert_eq!(player_command(Code::KeyA), None);
        assert_eq!(player_command(Code::Space), None);
    }
}
