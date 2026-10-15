// Точка сборки приложения: здесь настраиваем Tauri и запускаем окно.
// Позже сюда же добавятся команды, которые JS вызывает в Rust.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение Tauri");
}
