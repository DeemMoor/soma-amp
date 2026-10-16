// Сведения для окна «О программе»: версия, система, сборка, движок окна.
// Дату и коммит сборки подставляет build.rs (переменные SOMA_BUILD_DATE и SOMA_COMMIT).

use serde::Serialize;

/// То, что получает JS. `Serialize` превращает структуру в JSON-объект.
#[derive(Serialize)]
pub struct AppInfo {
    version: String,
    platform: String,
    build_date: &'static str,
    commit: &'static str,
    engine: String,
}

/// Человеческое название системы: "windows" → "Windows".
fn os_name(os: &str) -> &str {
    match os {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        other => other,
    }
}

/// Чем рисуется окно на каждой системе.
fn engine_name(os: &str) -> &'static str {
    match os {
        "windows" => "WebView2",
        "macos" => "WKWebView",
        _ => "WebKitGTK",
    }
}

/// Команда для JS: `invoke('app_info')`.
#[tauri::command]
pub fn app_info(app: tauri::AppHandle) -> AppInfo {
    let os = std::env::consts::OS;
    // версию движка узнаём у системы; не вышло — пишем только название
    let engine = match tauri::webview_version() {
        Ok(ver) => format!("{} {ver}", engine_name(os)),
        Err(_) => engine_name(os).to_string(),
    };
    AppInfo {
        version: app.package_info().version.to_string(),
        platform: format!("{} {}", os_name(os), std::env::consts::ARCH),
        build_date: env!("SOMA_BUILD_DATE"),
        commit: env!("SOMA_COMMIT"),
        engine,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_human_readable() {
        assert_eq!(os_name("windows"), "Windows");
        assert_eq!(os_name("macos"), "macOS");
        assert_eq!(os_name("freebsd"), "freebsd");
        assert_eq!(engine_name("linux"), "WebKitGTK");
    }
}
