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

/// Куда ведёт Help → «Сообщить о проблеме…».
const ISSUES_URL: &str = "https://github.com/DeemMoor/soma-amp/issues/new";
/// GitHub не принимает слишком длинные адреса — укладываемся с запасом.
const MAX_URL_LEN: usize = 7000;

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

/// Команда для JS: Help → «Сообщить о проблеме…». Открывает в браузере черновик
/// сообщения на GitHub, где уже есть версия, система и конец журнала.
#[tauri::command]
pub fn report_issue(app: tauri::AppHandle) -> Result<(), String> {
    let i = app_info(app);
    let facts = format!(
        "Soma Amp {} ({}), сборка {} · {}, {}",
        i.version, i.platform, i.build_date, i.commit, i.engine
    );
    let url = issue_url(&facts, &crate::logfile::tail(30));
    crate::links::open_in_browser(&url).map_err(|err| format!("не удалось открыть браузер: {err}"))
}

/// Адрес черновика: если не влезает, выкидываем самые старые строки журнала.
fn issue_url(facts: &str, log: &str) -> String {
    let make = |log: &str| {
        let body = format!(
            "**Что случилось:**\n\n\n**Как повторить:**\n\n\n---\n{facts}\n\n\
             <details><summary>Журнал</summary>\n\n```\n{log}\n```\n</details>\n"
        );
        format!("{ISSUES_URL}?body={}", encode(&body))
    };
    // сколько места осталось под журнал; строки берём с конца (самые свежие), пока влезают
    let mut room = MAX_URL_LEN.saturating_sub(make("").len());
    let mut kept: Vec<&str> = Vec::new();
    for line in log.lines().rev() {
        let cost = encode(line).len() + 3; // + перевод строки «%0A»
        if cost > room {
            break;
        }
        room -= cost;
        kept.push(line);
    }
    kept.reverse();
    make(&kept.join("\n"))
}

/// Кодирует текст для адреса: всё, кроме букв, цифр и `-_.~`, — в `%XX` по байтам UTF-8.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_is_percent_encoded() {
        assert_eq!(encode("a b&c"), "a%20b%26c");
        assert_eq!(encode("ё"), "%D1%91");
    }

    #[test]
    fn long_log_is_trimmed_from_the_start() {
        let log: String = (0..2000).map(|n| format!("строка {n}\n")).collect();
        let url = issue_url("Soma Amp", &log);
        assert!(url.len() <= MAX_URL_LEN);
        assert!(url.contains(&encode("строка 1999")));
        assert!(!url.contains(&encode("строка 0\n")));
    }

    #[test]
    fn names_are_human_readable() {
        assert_eq!(os_name("windows"), "Windows");
        assert_eq!(os_name("macos"), "macOS");
        assert_eq!(os_name("freebsd"), "freebsd");
        assert_eq!(engine_name("linux"), "WebKitGTK");
    }
}
