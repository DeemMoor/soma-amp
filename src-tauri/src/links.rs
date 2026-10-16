// Внешние ссылки (YouTube/Яндекс/Spotify у лайков, сайт SomaFM в меню) открываем
// в обычном браузере пользователя: у окна Tauri нет «новой вкладки», и ссылки
// с target="_blank" внутри него просто ничего не делают.

use std::process::Command;

/// Пропускаем только https-адреса. Иначе через эту команду можно было бы
/// открыть что угодно: file://, программу, системную ссылку.
fn is_allowed(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    // после https:// должен быть хотя бы адрес сайта, без пробелов и кавычек
    !rest.is_empty()
        && !url
            .chars()
            .any(|c| c.is_whitespace() || c == '"' || c.is_control())
}

/// Команда для JS: `invoke('open_url', { url })`.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !is_allowed(&url) {
        return Err(format!("ссылка не открыта (разрешены только https): {url}"));
    }
    open_in_browser(&url).map_err(|err| format!("не удалось открыть браузер: {err}"))
}

/// Windows сам открывает адрес браузером по умолчанию. Через url.dll, а не через `cmd /c start`:
/// так адрес не проходит через командную строку, и `&` в нём ничего не сломает.
#[cfg(windows)]
pub(crate) fn open_in_browser(url: &str) -> std::io::Result<()> {
    Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map(|_| ())
}

/// На macOS браузер по умолчанию открывает `open`.
#[cfg(target_os = "macos")]
pub(crate) fn open_in_browser(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

/// На Linux — `xdg-open` (есть во всех обычных рабочих окружениях).
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn open_in_browser(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_links_are_allowed() {
        assert!(is_allowed("https://somafm.com/"));
        assert!(is_allowed(
            "https://www.youtube.com/results?search_query=The%20XX%20Sunset"
        ));
        assert!(is_allowed("https://music.yandex.ru/search?text=a&b=c"));
    }

    #[test]
    fn other_schemes_are_rejected() {
        assert!(!is_allowed("http://somafm.com/"));
        assert!(!is_allowed("file:///C:/Windows/System32/calc.exe"));
        assert!(!is_allowed("C:\\Windows\\System32\\calc.exe"));
        assert!(!is_allowed("javascript:alert(1)"));
        assert!(!is_allowed("https://"));
        assert!(!is_allowed(""));
    }

    #[test]
    fn spaces_and_quotes_are_rejected() {
        assert!(!is_allowed("https://a.com/ x"));
        assert!(!is_allowed("https://a.com/\"x"));
        assert!(!is_allowed("https://a.com/\nx"));
    }
}
