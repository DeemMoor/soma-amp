# Как запустить работу над Soma Amp

## 1. Поставить инструменты (один раз, руками)

Открой **PowerShell** и по очереди:

1. **Visual Studio Build Tools** (компилятор C++ под Windows — Rust им линкует программы):
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
   ```
2. **Rust**:
   ```powershell
   winget install Rustlang.Rustup
   ```
   Закрой и открой PowerShell заново, проверь: `rustc -V` и `cargo -V`.
3. **WebView2** — на Windows 10/11 обычно уже стоит (это движок Edge). Если нет:
   ```powershell
   winget install Microsoft.EdgeWebView2Runtime
   ```
4. **Tauri CLI** (займёт несколько минут — компилируется):
   ```powershell
   cargo install tauri-cli --version "^2" --locked
   ```
   Проверка: `cargo tauri -V`.
5. **Git**, если ещё нет: `winget install Git.Git`.

## 2. Запустить Claude Code

```powershell
cd C:\projects\soma-amp
claude
```

Первое сообщение можно такое:

> Прочитай CLAUDE.md, ROADMAP.md и PROGRESS.md. Начинай с первого невыполненного пункта роадмэпа.

Дальше — по одному пункту за заход: «делай следующий пункт».

## 3. Где что лежит

- `ui/index.html` — копия плеера с сайта (фото-фон с сайта не нужен, уберём его в пункте 1.3) (версия от 2026-10-15, коммит сайта `aa41db1`).
- `CLAUDE.md` — правила для Claude.
- `ROADMAP.md` — план по шагам.
- `PROGRESS.md` — журнал: что сделано и чему научились.
