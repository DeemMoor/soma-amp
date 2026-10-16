# Разработка Soma Amp

## Сборка и проверки

Нужны [Rust](https://www.rust-lang.org/tools/install), Tauri CLI 2 и [системные зависимости Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
cargo install tauri-cli --version "^2" --locked
cd src-tauri
cargo tauri dev                 # запуск
cargo tauri build               # установщики — в src-tauri/target/release/bundle/
cargo fmt && cargo clippy -- -D warnings && cargo test
```

CI (`.github/workflows/ci.yml`) делает те же проверки на Windows, macOS и Linux. Dependabot раз в месяц присылает PR с обновлениями крейтов и GitHub Actions.

## Устройство

- `ui/index.html` — весь интерфейс (HTML/CSS/JS без сборщиков); тот же файл работает на сайте
- `src-tauri/src/lib.rs` — запуск, плагины, окно; в релизе UI раздаётся с `http://127.0.0.1:1430–1439` (с `tauri.localhost` SomaFM отвечает 403)
- `tray.rs`, `media_keys.rs` — трей и глобальные медиаклавиши
- `power.rs` — сон и выключение по таймеру; в отладочной сборке или при `SOMA_AMP_DRY_RUN` только пишет в журнал
- `settings.rs` — `settings.json`, атомарная запись
- `updates.rs` — автообновление (`tauri-plugin-updater`, GitHub Releases)
- `logfile.rs` — журнал ошибок `soma-amp.log` (Rust, JS, паники), ротация на 1 МБ
- `about.rs` — «О программе» и «Сообщить о проблеме»; дату и коммит сборки подставляет `build.rs`
- `autostart.rs`, `links.rs` — автозапуск, открытие ссылок в браузере (только https)

## Где лежат файлы

| | Windows | macOS | Linux |
|---|---|---|---|
| `settings.json` | `%APPDATA%\ru.deemmoor.somaamp\` | `~/Library/Application Support/ru.deemmoor.somaamp/` | `~/.config/ru.deemmoor.somaamp/` |
| `soma-amp.log` | `%LOCALAPPDATA%\ru.deemmoor.somaamp\logs\` | `~/Library/Logs/ru.deemmoor.somaamp/` | `~/.local/share/ru.deemmoor.somaamp/logs/` |

## Выпуск версии

1. Заметное для пользователя изменение — строка в `CHANGELOG.md`, раздел «Не выпущено», тем же коммитом.
2. Перед выпуском раздел переименовать в `## [X.Y.Z] — ГГГГ-ММ-ДД`, в `src-tauri/Cargo.toml` поднять `version` (единственное место версии).
3. Слить в `master`, поставить тег:

   ```sh
   git tag vX.Y.Z
   git push origin master vX.Y.Z
   ```

GitHub Actions соберёт установщики для всех ОС и создаст черновик релиза с описанием из `CHANGELOG.md`. Сборка остановится, если тег не совпадает с версией в `Cargo.toml` или в `CHANGELOG.md` нет раздела. После публикации черновика установленные программы видят обновление.

Обновления подписываются ключом из секретов репозитория `TAURI_SIGNING_PRIVATE_KEY` и `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Без этого ключа выпустить обновление для уже установленных копий нельзя.
