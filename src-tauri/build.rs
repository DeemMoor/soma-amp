// Сборочный скрипт: выполняется перед компиляцией приложения.
// Перечисляем наши команды, чтобы Tauri сгенерировал для них разрешения `allow-...`:
// без них страница с http://localhost:1430 (в установленной программе) не сможет их вызвать.
fn main() {
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
    ];
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(commands)),
    )
    .expect("не удалось подготовить сборку Tauri");
}
