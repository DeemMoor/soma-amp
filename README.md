# Soma Amp

Десктопный плеер интернет-радио [SomaFM](https://somafm.com/) в стиле Winamp 5 Modern.
Работает на Windows, Linux и macOS.

Веб-версия того же плеера: [deemmoor.ru/soma](https://deemmoor.ru/soma/).

> Soma Amp — неофициальный плеер, он не связан с SomaFM. SomaFM — независимое радио
> без рекламы, которое живёт на пожертвования слушателей.
> [Поддержать SomaFM](https://somafm.com/support/).

## Возможности

- 50+ каналов SomaFM, избранное, поиск, быстрый список Chill.
- Эквалайзер на 10 полос с пресетами, свой EQ для каждого канала, визуализация спектра.
- Цветовые темы, shade-режим (окно сворачивается в полоску), окно без рамки с прозрачным фоном.
- Лайки треков с поиском в YouTube, Яндекс Музыке и Spotify.
- Иконка в трее: играть/пауза, переключение каналов, работа в фоне.
- Глобальные медиаклавиши Play/Pause, Next, Prev, Stop — работают при свёрнутом окне.
- Таймер сна: по окончании — просто стоп, сон или выключение компьютера (с отменой за 60 секунд).
- «Поверх всех окон», автозапуск вместе с системой, один экземпляр программы.
- Автообновление: Help → «Проверить обновления…».

## Скачать

Последняя версия — на странице [Releases](https://github.com/DeemMoor/soma-amp/releases/latest).

| Система | Файл | Автообновление |
|---|---|---|
| Windows 10/11 | `Soma.Amp_x.y.z_x64-setup.exe` (или `.msi`) | да |
| macOS (Apple Silicon) | `Soma.Amp_aarch64.dmg` | да |
| macOS (Intel) | `Soma.Amp_x64.dmg` | да |
| Linux | `.AppImage` | да |
| Linux (Debian/Ubuntu, Fedora) | `.deb`, `.rpm` | нет — обновлять новым пакетом |

### Первая установка

Программа пока не подписана платным сертификатом, поэтому системы предупреждают при первом запуске:

- **Windows:** SmartScreen покажет «Windows защитила ваш компьютер» → «Подробнее» → «Выполнить в любом случае».
  `.exe` ставится для текущего пользователя без прав администратора, `.msi` — в Program Files.
- **macOS:** при первом открытии — правый клик по программе → «Открыть» → «Открыть».
  Если система пишет, что программа «повреждена», выполните в Терминале:
  `xattr -cr "/Applications/Soma Amp.app"`
- **Linux:** для `.AppImage` — `chmod +x Soma*.AppImage` и запуск.
  Глобальные медиаклавиши работают в X11; в Wayland — в зависимости от окружения.

## Горячие клавиши

| Клавиша | Действие |
|---|---|
| `X` / `C` / `V` | играть / пауза / стоп |
| `Z` / `B` | предыдущий / следующий канал |
| `S` | случайный канал |
| `Пробел` | играть / пауза |
| `↑` / `↓` | громкость |
| `M` | без звука |
| `L` | лайк текущему треку |
| `T` | таймер сна: выкл → 15 → 30 → 60 → 90 мин |
| `Alt+G` / `Alt+E` | показать/спрятать эквалайзер / плейлист |
| `Alt+X` | выход |
| Медиаклавиши | работают глобально, даже когда окно свёрнуто |

Клавиши работают и в русской раскладке.

## Где хранятся настройки

Файл `settings.json` (эквалайзер, тема, избранное, лайки, положение окна):

- Windows: `%APPDATA%\ru.deemmoor.somaamp\`
- macOS: `~/Library/Application Support/ru.deemmoor.somaamp/`
- Linux: `~/.config/ru.deemmoor.somaamp/`

Чтобы сохранить лайки и настройки, достаточно скопировать этот файл.

## Сборка из исходников

Нужны [Rust](https://www.rust-lang.org/tools/install) и Tauri CLI 2
(плюс [системные зависимости Tauri](https://v2.tauri.app/start/prerequisites/) для вашей ОС):

```sh
cargo install tauri-cli --version "^2" --locked
cargo tauri dev     # запуск в режиме разработки
cargo tauri build   # установщики — в src-tauri/target/release/bundle/
```

Проверки перед коммитом (в папке `src-tauri`):

```sh
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## Как выпустить новую версию

1. Поднять версию в `src-tauri/tauri.conf.json` и `src-tauri/Cargo.toml`.
2. Закоммитить в `master` и поставить тег: `git tag v0.1.3 && git push origin master v0.1.3`.
3. GitHub Actions (`.github/workflows/release.yml`) соберёт установщики для всех систем
   и создаст **черновик** релиза.
4. Проверить файлы в черновике и нажать **Publish release** — после этого
   установленные программы увидят обновление.

Обновления подписываются ключом. Секреты репозитория (Settings → Secrets and variables → Actions):
`TAURI_SIGNING_PRIVATE_KEY` и `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
Если ключ потерять, выпускать обновления для уже установленных копий станет невозможно.

## Устройство

- `ui/index.html` — весь интерфейс плеера (HTML/CSS/JS без сборщиков); тот же файл работает и на сайте.
- `src-tauri/src/` — Rust-часть: трей (`tray.rs`), медиаклавиши (`media_keys.rs`),
  сон/выключение (`power.rs`), настройки (`settings.rs`), автозапуск (`autostart.rs`),
  ссылки (`links.rs`), обновления (`updates.rs`).
- Стек: [Tauri 2](https://v2.tauri.app/) + Rust.

Журнал разработки с объяснениями — `PROGRESS.md`, план — `ROADMAP.md`.

## Автор

Deem Moor — [github.com/DeemMoor](https://github.com/DeemMoor)
