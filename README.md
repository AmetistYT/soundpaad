# SoundPad

Sound pad приложение для Linux с поддержкой виртуального микрофона.

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![GTK4](https://img.shields.io/badge/GTK4-0.9-blue?logo=gtk)
![License](https://img.shields.io/badge/License-MIT-green)

## Возможности

- **Виртуальный микрофон** — перенаправление звука в виртуальный микрофон (SoundPad_Mic) для использования в Discord, OBS и других приложениях
- **Наложение голоса** — реальный микрофон проходит через виртуальный, звуки саундпада накладываются поверх
- **Магазин звуков** — поиск и скачивание звуков с myinstants.com
- **Хоткеи** — назначение горячих клавиш на звуки
- **Воспроизведение** — звук в колонках + виртуальный микрофон одновременно, или только в микрофон

## Установка

### AUR (Arch Linux)

```bash
yay -S soundpaad-bin
```

### Из исходников

```bash
git clone https://github.com/AmetistYT/soundpaad.git
cd soundpaad
cargo build --release
sudo cp target/release/soundpaad /usr/bin/
sudo cp com.soundpaad.app.desktop /usr/share/applications/
sudo cp com.soundpaad.app.svg /usr/share/icons/hicolor/scalable/apps/
```

## Зависимости

- GTK4 + libadwaita
- GStreamer
- PulseAudio / PipeWire (pactl)
- OpenSSL

### Arch Linux

```bash
sudo pacman -S gtk4 libadwaita gstreamer gst-plugins-base gst-plugins-good pulseaudio openssl
```

## Использование

1. Запустите SoundPad
2. В настройках нажмите **Создать** для создания виртуального микрофона
3. Выберите реальный микрофон в разделе **Микрофон для наложения**
4. В приложениях (Discord, OBS и т.д.) выберите **SoundPad_Mic** как микрофон
5. Скачивайте звуки из магазина и назначайте хоткеи

## Технологии

- **Rust** — язык программирования
- **GTK4 + libadwaita** — UI
- **GStreamer** — аудио воспроизведение
- **SQLite** — хранение данных
- **reqwest + scraper** — парсинг myinstants.com

## Лицензия

MIT
