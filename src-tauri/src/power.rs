// Действия с компьютером по окончании таймера сна: сон и выключение.
// JS сам показывает предупреждение с отменой за 60 секунд и только потом вызывает
// команду `system_action`. В отладочной сборке (cargo tauri dev) команда ничего
// не выполняет, а только пишет, что сделала бы, — это «сухой режим».

use std::process::Command;

#[derive(Debug, PartialEq)]
enum PowerAction {
    Sleep,
    Shutdown,
}

impl PowerAction {
    /// Разбирает строку из JS. Всё, кроме "sleep" и "shutdown", — ошибка.
    fn parse(name: &str) -> Result<Self, String> {
        match name {
            "sleep" => Ok(Self::Sleep),
            "shutdown" => Ok(Self::Shutdown),
            other => Err(format!("неизвестное действие: «{other}»")),
        }
    }

    /// Какую программу запустить и с какими аргументами.
    fn command_line(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            // /t 0 — без системной задержки: свои 60 секунд с отменой уже были в плеере
            Self::Shutdown => ("shutdown", &["/s", "/t", "0"]),
            // именно сон: SetSuspendState('Suspend'). Популярный rundll32 powrprof.dll,SetSuspendState
            // при включённой гибернации вместо сна делает гибернацию
            Self::Sleep => (
                "powershell",
                &[
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Add-Type -AssemblyName System.Windows.Forms; \
                     [System.Windows.Forms.Application]::SetSuspendState('Suspend', $false, $false)",
                ],
            ),
        }
    }
}

/// Сухой режим: в отладочной сборке всегда, в релизе — если задана переменная SOMA_AMP_DRY_RUN.
fn is_dry_run() -> bool {
    cfg!(debug_assertions) || std::env::var_os("SOMA_AMP_DRY_RUN").is_some()
}

/// Команда для JS: `invoke('system_action', { action: 'sleep' | 'shutdown' })`.
/// Возвращает текст для показа на дисплее плеера (в сухом режиме — что выполнили бы).
#[tauri::command]
pub fn system_action(action: String) -> Result<String, String> {
    let action = PowerAction::parse(&action)?;
    let (program, args) = action.command_line();
    let line = format!("{program} {}", args.join(" "));

    if is_dry_run() {
        println!("[сухой режим] выполнил бы: {line}");
        let what = match action {
            PowerAction::Sleep => "сон",
            PowerAction::Shutdown => "выключение",
        };
        return Ok(format!("Сухой режим: {what} пропущен"));
    }

    let mut cmd = Command::new(program);
    cmd.args(args);
    hide_console_window(&mut cmd);
    cmd.spawn()
        .map_err(|err| format!("не удалось запустить «{line}»: {err}"))?;
    Ok(String::new())
}

/// Чтобы при запуске powershell/shutdown не мелькало чёрное окно консоли.
#[cfg(windows)]
fn hide_console_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console_window(_cmd: &mut Command) {}

// Тесты проверяют только разбор и командные строки — system_action здесь
// специально НЕ вызываем: `cargo test --release` выключил бы компьютер.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_actions() {
        assert_eq!(PowerAction::parse("sleep"), Ok(PowerAction::Sleep));
        assert_eq!(PowerAction::parse("shutdown"), Ok(PowerAction::Shutdown));
    }

    #[test]
    fn rejects_unknown_actions() {
        assert!(PowerAction::parse("reboot").is_err());
        assert!(PowerAction::parse("stop").is_err());
        assert!(PowerAction::parse("").is_err());
    }

    #[test]
    fn shutdown_is_immediate_power_off() {
        assert_eq!(
            PowerAction::Shutdown.command_line(),
            ("shutdown", &["/s", "/t", "0"][..])
        );
    }

    #[test]
    fn sleep_suspends_instead_of_hibernating() {
        let (program, args) = PowerAction::Sleep.command_line();
        assert_eq!(program, "powershell");
        assert!(args.iter().any(|a| a.contains("SetSuspendState('Suspend'")));
    }

    #[test]
    fn debug_build_is_dry_run() {
        // тесты собираются в отладочном режиме — значит, сухой режим обязан быть включён
        if cfg!(debug_assertions) {
            assert!(is_dry_run());
        }
    }
}
