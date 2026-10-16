// Сборочный скрипт: выполняется перед компиляцией приложения.
// Перечисляем наши команды, чтобы Tauri сгенерировал для них разрешения `allow-...`:
// без них страница с http://localhost:1430 (в установленной программе) не сможет их вызвать.
use std::process::Command;

/// Спрашивает у git одну строку (дату или хеш коммита). Нет git — `None`.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

fn main() {
    // дата и короткий хеш последнего коммита — для окна «О программе»
    let date = git(&["log", "-1", "--format=%cs"]).unwrap_or_else(|| "?".into());
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "?".into());
    println!("cargo:rustc-env=SOMA_BUILD_DATE={date}");
    println!("cargo:rustc-env=SOMA_COMMIT={commit}");
    // пересобрать, когда появился новый коммит
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/heads");

    let commands = &[
        "system_action",
        "autostart_enabled",
        "set_autostart",
        "settings_set",
        "settings_import",
        "open_url",
        "check_update",
        "install_update",
        "write_log",
        "app_info",
        "report_issue",
        "open_log",
    ];
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(commands)),
    )
    .expect("не удалось подготовить сборку Tauri");
}
