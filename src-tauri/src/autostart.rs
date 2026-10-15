// Автозапуск вместе с Windows. Плагин прописывает программу в автозагрузку
// текущего пользователя (реестр HKCU\Software\Microsoft\Windows\CurrentVersion\Run).
// Состояние всегда спрашиваем у системы, а не храним сами: автозагрузку могут
// выключить и в Диспетчере задач, и галочка в меню не должна врать.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Команда для JS: включён ли автозапуск сейчас.
#[tauri::command]
pub fn autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|err| err.to_string())
}

/// Команда для JS: включить или выключить автозапуск.
#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|err| err.to_string())
}
