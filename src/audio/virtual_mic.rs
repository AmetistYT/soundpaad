use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioServer {
    PipeWire,
    PulseAudio,
}

const VIRTUAL_MIC_NAME: &str = "SoundPad_Mic";
const VIRTUAL_MIC_DESC: &str = "SoundPad Virtual Microphone";

pub struct VirtualMic {
    server: AudioServer,
    sink_id: Option<u32>,
    loopback_id: Option<u32>,
}

impl VirtualMic {
    pub fn detect_server() -> Result<AudioServer> {
        let output = Command::new("pactl")
            .args(["info"])
            .output()
            .context("Не удалось выполнить pactl info. Установлен ли PulseAudio/PipeWire?")?;

        let info = String::from_utf8_lossy(&output.stdout);
        if info.contains("PipeWire") {
            Ok(AudioServer::PipeWire)
        } else if info.contains("PulseAudio") {
            Ok(AudioServer::PulseAudio)
        } else {
            Ok(AudioServer::PipeWire)
        }
    }

    pub fn new() -> Result<Self> {
        let server = Self::detect_server()?;
        Ok(Self {
            server,
            sink_id: None,
            loopback_id: None,
        })
    }

    pub fn server(&self) -> AudioServer {
        self.server
    }

    pub fn is_active(&self) -> bool {
        self.sink_id.is_some()
    }

    pub fn sink_name(&self) -> &str {
        VIRTUAL_MIC_NAME
    }

    pub fn create(&mut self) -> Result<()> {
        if self.sink_id.is_some() {
            return Ok(());
        }

        let sink_name = format!("sink_name={}", VIRTUAL_MIC_NAME);
        let sink_desc = format!("sink_properties=device.description=\"{}\"", VIRTUAL_MIC_DESC);

        // Создаём null sink
        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-null-sink",
                &sink_name,
                "sink_format=s16le",
                "sink_rate=48000",
                "sink_channels=2",
                &sink_desc,
            ])
            .output()
            .context("Не удалось создать null sink")?;

        if !output.status.success() {
            anyhow::bail!(
                "Ошибка создания null sink: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let id_str = String::from_utf8_lossy(&output.stdout);
        let sink_id: u32 = id_str.trim().parse().context("Не удалось распарсить ID sink")?;
        self.sink_id = Some(sink_id);

        // Создаём loopback — перенаправление звука из виртуального микрофона в реальные колонки
        let monitor_name = format!("{}\\.monitor", VIRTUAL_MIC_NAME);
        let loopback_output = Command::new("pactl")
            .args([
                "load-module",
                "module-loopback",
                &format!("source={}", monitor_name),
                "latency_msec=1",
            ])
            .output()
            .context("Не удалось создать loopback")?;

        if loopback_output.status.success() {
            let lb_id_str = String::from_utf8_lossy(&loopback_output.stdout);
            if let Ok(lb_id) = lb_id_str.trim().parse::<u32>() {
                self.loopback_id = Some(lb_id);
            }
        }

        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        if let Some(lb_id) = self.loopback_id.take() {
            let _ = Command::new("pactl")
                .args(["unload-module", &lb_id.to_string()])
                .output();
        }

        if let Some(sink_id) = self.sink_id.take() {
            let output = Command::new("pactl")
                .args(["unload-module", &sink_id.to_string()])
                .output()
                .context("Не удалось выгрузить null sink")?;

            if !output.status.success() {
                anyhow::bail!(
                    "Ошибка выгрузки null sink: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(())
    }

    pub fn get_default_sink() -> Result<String> {
        let output = Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .context("Не удалось получить default sink")?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn list_sources() -> Result<Vec<String>> {
        let output = Command::new("pactl")
            .args(["list", "short", "sources"])
            .output()
            .context("Не удалось получить список sources")?;

        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text
            .lines()
            .filter_map(|l| l.split_whitespace().nth(1).map(String::from))
            .filter(|s| !s.ends_with(".monitor"))
            .collect())
    }
}

impl Drop for VirtualMic {
    fn drop(&mut self) {
        let _ = self.destroy();
    }
}
