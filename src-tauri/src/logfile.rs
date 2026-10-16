// Журнал ошибок в файл soma-amp.log (папка журналов приложения; на Windows —
// %LOCALAPPDATA%\ru.deemmoor.somaamp\logs). В установленной программе нет консоли,
// и без файла сообщения об ошибках пропадали бы бесследно.
// Пишем через макросы log_error! / log_info! — как println!, только в журнал.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Файл больше этого размера переименовываем в .old и начинаем новый.
const MAX_SIZE: u64 = 1024 * 1024;

/// Путь к журналу; задаётся один раз при запуске (`init`).
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
/// Чтобы две записи из разных потоков не перемешались в одной строке.
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Запомнить папку журналов. До вызова `init` записи идут только в консоль.
pub fn init(dir: PathBuf) {
    // журнал — вспомогательная вещь: если папку создать не вышло, программа работает дальше
    let _ = fs::create_dir_all(&dir);
    let _ = LOG_PATH.set(dir.join("soma-amp.log"));

    // Паника — аварийная остановка Rust-кода (ошибка в программе, а не в данных).
    // Сначала пишем её в журнал, потом отдаём стандартному обработчику: без этого
    // от падения установленной программы не осталось бы и следа.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        write("ПАДЕНИЕ", &info.to_string());
        default_hook(info);
    }));
}

/// Записать строку в журнал (и в консоль — в режиме разработки её видно).
pub fn write(level: &str, message: &str) {
    eprintln!("[{level}] {message}");
    let Some(path) = LOG_PATH.get() else {
        return;
    };
    let _guard = WRITE_LOCK.lock();
    if fs::metadata(path).is_ok_and(|m| m.len() > MAX_SIZE) {
        let _ = fs::rename(path, path.with_extension("log.old"));
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let line = format!("{} [{level}] {message}\n", format_utc(secs));
    // ошибку записи в журнал некуда сообщить — просто пропускаем строку
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

/// Секунды с 1970-01-01 → «2026-10-16 12:34:56 UTC» (без внешних крейтов).
fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rest = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Номер дня от 1970-01-01 → (год, месяц, день). Известный алгоритм Говарда Хиннанта.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Ошибка в журнал: `log_error!("не удалось {}", что)`.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::logfile::write("ОШИБКА", &format!($($arg)*)) };
}

/// Просто сообщение в журнал: `log_info!("...")`.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::logfile::write("инфо", &format!($($arg)*)) };
}

/// Команда для JS: записать в журнал ошибку из страницы (window.onerror и т.п.).
#[tauri::command]
pub fn write_log(message: String) {
    // ограничиваем длину: одна огромная строка не должна раздуть журнал
    let short: String = message.chars().take(2000).collect();
    write("JS", &short);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_dates() {
        assert_eq!(format_utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_utc(951_782_400), "2000-02-29 00:00:00 UTC");
        assert_eq!(format_utc(1_704_067_199), "2023-12-31 23:59:59 UTC");
        assert_eq!(format_utc(1_704_067_200), "2024-01-01 00:00:00 UTC");
    }

    #[test]
    fn leap_years() {
        assert_eq!(civil_from_days(59), (1970, 3, 1)); // 1970 — не високосный
        assert_eq!(civil_from_days(11_016), (2000, 2, 29)); // 2000 — високосный
        assert_eq!(civil_from_days(47_541), (2100, 3, 1)); // 2100 — нет (делится на 100)
    }

    #[test]
    fn panic_goes_to_log() {
        // единственный тест, который вызывает init: журнал — во временной папке
        let dir = std::env::temp_dir().join(format!("soma-amp-test-{}", std::process::id()));
        init(dir.clone());
        // паника в отдельном потоке роняет только этот поток, а не тест
        let _ = std::thread::spawn(|| panic!("проверка журнала")).join();
        let text = fs::read_to_string(dir.join("soma-amp.log")).unwrap_or_default();
        let _ = fs::remove_dir_all(&dir);
        assert!(text.contains("[ПАДЕНИЕ]") && text.contains("проверка журнала"));
    }
}
