mod audio;
mod db;
mod shop;
mod ui;

use adw::prelude::*;
use adw::Application;
use gtk4::gio;
use gtk4::glib;
use std::cell::RefCell;
use std::rc::Rc;

use audio::player::AudioPlayer;
use audio::virtual_mic::VirtualMic;
use db::repository::Database;

const APP_ID: &str = "com.soundpaad.app";

pub struct AppState {
    pub db: Database,
    pub player: AudioPlayer,
    pub virtual_mic: VirtualMic,
}

fn main() -> glib::ExitCode {
    gstreamer::init().expect("Не удалось инициализировать GStreamer");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let db = Database::open_default().expect("Не удалось открыть базу данных");
    let virtual_mic = VirtualMic::new().expect("Не удалось определить аудио сервер");
    let sink_name = virtual_mic.sink_name().to_string();
    let player = AudioPlayer::new(&sink_name).expect("Не удалось инициализировать аудио плеер");

    let state = Rc::new(RefCell::new(AppState {
        db,
        player,
        virtual_mic,
    }));

    let _ui = ui::app::AppUI::build(app, state.clone());
}
