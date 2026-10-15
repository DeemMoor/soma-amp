// Настройки плеера в файле settings.json (папка конфигурации приложения,
// на Windows — %APPDATA%\ru.deemmoor.somaamp\). Это простой словарь «ключ → значение»:
// какие настройки бывают (эквалайзер, тема, избранное, лайки…), решает JS-плеер.
//
// Плеер читает настройки синхронно при старте, поэтому Rust ещё до запуска страницы
// кладёт их в window.__SOMA_SETTINGS__ (см. init_script), а запись идёт командой settings_set.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, State};

/// Все настройки: ключ → любое JSON-значение.
pub type Settings = Map<String, Value>;

/// Настройки в памяти + путь к файлу. Tauri хранит его как общее состояние приложения.
pub struct SettingsStore {
    path: PathBuf,
    /// Был ли файл при запуске. Нет — значит, первый запуск: JS перенесёт старые настройки.
    existed: bool,
    /// Mutex — чтобы две команды из JS не меняли настройки одновременно.
    data: Mutex<Settings>,
}

impl SettingsStore {
    /// Открывает файл настроек приложения (или начинает с пустых, если файла нет).
    pub fn open(app: &AppHandle) -> tauri::Result<Self> {
        let path = app.path().app_config_dir()?.join("settings.json");
        let loaded = load(&path)?;
        Ok(Self {
            path,
            existed: loaded.is_some(),
            data: Mutex::new(loaded.unwrap_or_default()),
        })
    }

    /// JS-код, который выполнится в окне раньше страницы и положит туда настройки.
    pub fn init_script(&self) -> Result<String, String> {
        let data = self.lock()?;
        let json = serde_json::to_string(&*data).map_err(|err| err.to_string())?;
        Ok(format!(
            "window.__SOMA_SETTINGS__ = {{ existed: {}, data: {json} }};",
            self.existed
        ))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Settings>, String> {
        self.data
            .lock()
            .map_err(|_| "настройки недоступны: прошлая запись упала".to_string())
    }
}

/// Читает настройки из файла. Файла нет — `None`.
/// Файл битый — откладываем его как `settings.json.broken` (не затираем!) и тоже `None`.
pub fn load(path: &Path) -> io::Result<Option<Settings>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    match serde_json::from_str(&text) {
        Ok(settings) => Ok(Some(settings)),
        Err(err) => {
            eprintln!(
                "настройки: файл {} повреждён ({err}), откладываю его",
                path.display()
            );
            fs::rename(path, path.with_extension("json.broken"))?;
            Ok(None)
        }
    }
}

/// Записывает настройки. Сначала во временный файл, потом подменяем им старый:
/// если компьютер выключится посреди записи, старый файл останется целым.
pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(settings)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

/// Меняет одну настройку. `null` из JS значит «удалить».
fn apply(settings: &mut Settings, key: String, value: Value) {
    if value.is_null() {
        settings.remove(&key);
    } else {
        settings.insert(key, value);
    }
}

/// Добавляет только те настройки, которых ещё нет: перенос старых не затирает новые.
fn merge_missing(settings: &mut Settings, entries: Settings) {
    for (key, value) in entries {
        settings.entry(key).or_insert(value);
    }
}

/// Команда для JS: `invoke('settings_set', { key, value })`.
#[tauri::command]
pub fn settings_set(store: State<SettingsStore>, key: String, value: Value) -> Result<(), String> {
    let mut data = store.lock()?;
    apply(&mut data, key, value);
    save(&store.path, &data).map_err(|err| format!("не удалось сохранить настройки: {err}"))
}

/// Команда для JS: перенос старых настроек из localStorage при первом запуске.
#[tauri::command]
pub fn settings_import(store: State<SettingsStore>, entries: Settings) -> Result<(), String> {
    let mut data = store.lock()?;
    merge_missing(&mut data, entries);
    save(&store.path, &data).map_err(|err| format!("не удалось сохранить настройки: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Отдельная временная папка на каждый тест, чтобы тесты не мешали друг другу.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("soma-amp-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn sample() -> Settings {
        let mut s = Settings::new();
        s.insert("skin".into(), json!("silver"));
        s.insert("vol".into(), json!(70));
        s.insert("favs".into(), json!(["groovesalad", "dronezone"]));
        s.insert("eq".into(), json!({ "on": true, "bands": [0, 3, -2] }));
        s
    }

    #[test]
    fn missing_file_means_no_settings() {
        let dir = temp_dir("missing");
        assert_eq!(load(&dir.join("settings.json")).unwrap(), None);
    }

    #[test]
    fn save_then_load_returns_the_same() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("settings.json");
        save(&path, &sample()).unwrap();
        assert_eq!(load(&path).unwrap(), Some(sample()));
        assert!(
            !path.with_extension("json.tmp").exists(),
            "временный файл не убран"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn save_creates_missing_folders() {
        let dir = temp_dir("folders");
        let path = dir.join("a").join("b").join("settings.json");
        save(&path, &sample()).unwrap();
        assert!(path.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn broken_file_is_kept_aside_not_overwritten() {
        let dir = temp_dir("broken");
        let path = dir.join("settings.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "{ это не json").unwrap();
        assert_eq!(load(&path).unwrap(), None);
        let broken = path.with_extension("json.broken");
        assert_eq!(fs::read_to_string(broken).unwrap(), "{ это не json");
        assert!(!path.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn null_removes_a_setting() {
        let mut s = sample();
        apply(&mut s, "vol".into(), Value::Null);
        assert!(!s.contains_key("vol"));
        apply(&mut s, "vol".into(), json!(30));
        assert_eq!(s["vol"], json!(30));
    }

    #[test]
    fn import_does_not_overwrite_existing() {
        let mut s = sample();
        let mut old = Settings::new();
        old.insert("skin".into(), json!("old-theme"));
        old.insert("likes".into(), json!([{ "title": "Song" }]));
        merge_missing(&mut s, old);
        assert_eq!(s["skin"], json!("silver"));
        assert_eq!(s["likes"], json!([{ "title": "Song" }]));
    }
}
