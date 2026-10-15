// Автообновление (плагин tauri-plugin-updater). Программа скачивает с GitHub файл
// latest.json из последнего релиза, сравнивает версию со своей и, если есть новее,
// скачивает установщик. Каждое обновление подписано нашим ключом: программа проверит
// подпись публичным ключом из tauri.conf.json и не поставит подделку.

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Что рассказать пользователю о найденном обновлении.
#[derive(Serialize)]
pub struct UpdateInfo {
    version: String,
    notes: Option<String>,
}

/// Команда для JS: есть ли новая версия. `None` — у тебя последняя.
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let update = app
        .updater()
        .map_err(|err| err.to_string())?
        .check()
        .await
        .map_err(|err| format!("не удалось проверить обновления: {err}"))?;
    Ok(update.map(|u| UpdateInfo {
        version: u.version,
        notes: u.body,
    }))
}

/// Команда для JS: скачать и поставить новую версию, потом перезапустить программу.
/// На Windows установщик сам закрывает программу и запускает новую версию.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = app
        .updater()
        .map_err(|err| err.to_string())?
        .check()
        .await
        .map_err(|err| format!("не удалось проверить обновления: {err}"))?
        .ok_or("обновлений нет")?;
    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|err| format!("не удалось установить обновление: {err}"))?;
    app.restart()
}
