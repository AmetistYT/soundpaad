use anyhow::{Context, Result};
use gstreamer::prelude::*;
use gstreamer::{Pipeline, State};
use gtk4::glib;
use std::collections::HashMap;
use std::path::Path;

pub struct AudioPlayer {
    pipelines: HashMap<i64, Pipeline>,
    virtual_mic_sink: String,
    default_sink: String,
    play_to_speakers: bool,
    real_mic_source: String,
    mic_passthrough: Option<Pipeline>,
}

impl AudioPlayer {
    pub fn new(virtual_mic_sink: &str) -> Result<Self> {
        gstreamer::init().context("Не удалось инициализировать GStreamer")?;

        let default_sink = crate::audio::virtual_mic::VirtualMic::get_default_sink()
            .unwrap_or_else(|_| "autoaudiosink".to_string());

        Ok(Self {
            pipelines: HashMap::new(),
            virtual_mic_sink: virtual_mic_sink.to_string(),
            default_sink,
            play_to_speakers: true,
            real_mic_source: String::new(),
            mic_passthrough: None,
        })
    }

    pub fn play(&mut self, track_id: i64, file_path: &str, volume: f64) -> Result<()> {
        self.stop(track_id).ok();

        let path = Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("Файл не найден: {}", file_path);
        }

        let canonical = path.canonicalize()?;

        let launch_str = if self.play_to_speakers {
            format!(
                "filesrc location=\"{}\" ! decodebin ! audioconvert ! audioresample ! volume volume={} ! tee name=t \
                 t. ! queue ! pulsesink device=\"{}\" sync=false \
                 t. ! queue ! pulsesink device=\"{}\" sync=false",
                canonical.display(),
                volume,
                self.default_sink,
                self.virtual_mic_sink,
            )
        } else {
            format!(
                "filesrc location=\"{}\" ! decodebin ! audioconvert ! audioresample ! volume volume={} ! pulsesink device=\"{}\" sync=false",
                canonical.display(),
                volume,
                self.virtual_mic_sink,
            )
        };

        eprintln!("Play: pipeline={}", launch_str);

        let pipeline = gstreamer::parse::launch(&launch_str)
            .map_err(|e| { eprintln!("Play: ошибка создания pipeline: {}", e); e })?;

        let pipeline_clone = pipeline.clone().downcast::<Pipeline>().unwrap_or_else(|_| {
            Pipeline::new()
        });
        let bus = pipeline.bus().unwrap();
        let _watch_guard = bus.add_watch(move |_bus, msg| {
            use gstreamer::MessageView;
            match msg.view() {
                MessageView::Error(err) => {
                    eprintln!("Play: pipeline error: {} ({})", err.error(), err.debug().unwrap_or_default());
                    pipeline_clone.set_state(State::Null).ok();
                }
                MessageView::Eos(_) => {
                    pipeline_clone.set_state(State::Null).ok();
                }
                _ => {}
            }
            glib::ControlFlow::Continue
        })?;

        pipeline.set_state(State::Playing)
            .map_err(|e| { eprintln!("Play: ошибка запуска: {}", e); e })?;

        eprintln!("Play: запущен track_id={}", track_id);

        let pipeline_downcast = pipeline.downcast::<Pipeline>()
            .map_err(|_| anyhow::anyhow!("Pipeline downcast failed"))?;
        self.pipelines.insert(track_id, pipeline_downcast);
        Ok(())
    }

    pub fn stop(&mut self, track_id: i64) -> Result<()> {
        if let Some(pipeline) = self.pipelines.remove(&track_id) {
            pipeline.set_state(State::Null)?;
        }
        Ok(())
    }

    pub fn stop_all(&mut self) {
        for (_, pipeline) in self.pipelines.drain() {
            let _ = pipeline.set_state(State::Null);
        }
    }

    pub fn is_playing(&self, track_id: i64) -> bool {
        self.pipelines.get(&track_id)
            .map(|p| p.current_state() == State::Playing)
            .unwrap_or(false)
    }

    pub fn set_virtual_mic_sink(&mut self, sink: &str) {
        self.virtual_mic_sink = sink.to_string();
    }

    pub fn set_default_sink(&mut self, sink: &str) {
        self.default_sink = sink.to_string();
    }

    pub fn virtual_mic_sink(&self) -> &str {
        &self.virtual_mic_sink
    }

    pub fn default_sink(&self) -> &str {
        &self.default_sink
    }

    pub fn set_play_to_speakers(&mut self, val: bool) {
        self.play_to_speakers = val;
    }

    pub fn play_to_speakers(&self) -> bool {
        self.play_to_speakers
    }

    pub fn set_real_mic_source(&mut self, source: &str) {
        self.real_mic_source = source.to_string();
        self.start_mic_passthrough();
    }

    pub fn real_mic_source(&self) -> &str {
        &self.real_mic_source
    }

    /// Запускает прохождение реального микрофона через виртуальный микрофон
    /// Реальный микрофон -> pulsesink (вирт. микр.)
    /// Звуки саундпада тоже идут в вирт. микр., поэтому они смешиваются
    pub fn start_mic_passthrough(&mut self) {
        self.stop_mic_passthrough();

        if self.real_mic_source.is_empty() || self.virtual_mic_sink.is_empty() {
            eprintln!("Mic passthrough: не задан источник или sink (source={}, sink={})", self.real_mic_source, self.virtual_mic_sink);
            return;
        }

        let launch_str = format!(
            "pulsesrc device=\"{}\" ! audioconvert ! audioresample ! volume volume=1.0 ! pulsesink device=\"{}\" sync=false",
            self.real_mic_source,
            self.virtual_mic_sink,
        );

        match gstreamer::parse::launch(&launch_str) {
            Ok(pipeline) => {
                let pipeline = pipeline.downcast::<Pipeline>().unwrap_or_else(|_| Pipeline::new());
                match pipeline.set_state(State::Playing) {
                    Ok(_) => {
                        eprintln!("Mic passthrough: запущен ({} -> {})", self.real_mic_source, self.virtual_mic_sink);
                        self.mic_passthrough = Some(pipeline);
                    }
                    Err(e) => eprintln!("Mic passthrough: ошибка запуска: {}", e),
                }
            }
            Err(e) => eprintln!("Mic passthrough: ошибка создания pipeline: {}", e),
        }
    }

    pub fn stop_mic_passthrough(&mut self) {
        if let Some(pipeline) = self.mic_passthrough.take() {
            let _ = pipeline.set_state(State::Null);
        }
    }

    pub fn is_mic_passthrough_active(&self) -> bool {
        self.mic_passthrough.is_some()
    }
}
