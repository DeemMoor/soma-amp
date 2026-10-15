// В релизной сборке не показываем лишнее чёрное окно консоли. НЕ УДАЛЯТЬ!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    soma_amp_lib::run();
}
