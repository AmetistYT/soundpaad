use adw::prelude::*;
use gstreamer::prelude::*;
use gtk4::glib;
use gtk4::{Box as GtkBox, Button, Entry, Label, ListBox, Orientation, ProgressBar, ScrolledWindow, Stack};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use crate::db::models::ShopSound;
use crate::shop::myinstants::MyInstantsClient;
use crate::AppState;
use super::soundpad::SoundpadPage;

enum ShopAction {
    Sounds(Vec<ShopSound>),
    Error(String),
    Downloaded(String, String),
}

pub struct ShopPage {
    container: gtk4::Box,
    listbox: ListBox,
    stack: Stack,
    progress: ProgressBar,
    state: Rc<RefCell<AppState>>,
    _soundpad: Rc<SoundpadPage>,
}

impl ShopPage {
    pub fn new(state: Rc<RefCell<AppState>>, soundpad: Rc<SoundpadPage>) -> Self {
        let container = gtk4::Box::new(Orientation::Vertical, 8);
        container.set_margin_start(12); container.set_margin_end(12);
        container.set_margin_top(12); container.set_margin_bottom(12);

        let search_box = GtkBox::new(Orientation::Horizontal, 6);
        let search_entry = Entry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some("Поиск звуков..."));
        let search_btn = Button::with_label("Найти");
        search_box.append(&search_entry); search_box.append(&search_btn);
        container.append(&search_box);

        let categories_label = Label::new(Some("Категории:"));
        let cat_box = GtkBox::new(Orientation::Horizontal, 4);
        cat_box.append(&categories_label);

        let cat_scroll = ScrolledWindow::new();
        cat_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Never);
        cat_scroll.set_hexpand(true);
        let cat_flow = gtk4::FlowBox::new();
        cat_flow.set_selection_mode(gtk4::SelectionMode::None);
        cat_flow.set_column_spacing(4);

        let (sender, receiver) = mpsc::channel::<ShopAction>();
        let sender = Arc::new(Mutex::new(sender));

        let default_cats = vec![
            ("Memes", "/en/categories/memes/"), ("Games", "/en/categories/games/"),
            ("Anime", "/en/categories/anime%20&%20manga/"), ("Music", "/en/categories/music/"),
            ("FX", "/en/categories/sound%20effects/"), ("Viral", "/en/categories/viral/"),
        ];

        let stack = Stack::new();
        let progress = ProgressBar::new();
        progress.set_fraction(0.0);

        for (name, path) in default_cats {
            let btn = Button::with_label(name);
            btn.add_css_class("pill"); btn.add_css_class("flat");
            let s_clone = sender.clone();
            let p_owned = path.to_string();
            let st_ref = stack.clone();
            let pr_ref = progress.clone();
            btn.connect_clicked(move |_| {
                st_ref.set_visible_child_name("loading");
                pr_ref.set_fraction(0.1);
                let s = s_clone.clone(); let p = p_owned.clone();
                std::thread::spawn(move || {
                    if let Ok(client) = MyInstantsClient::new() {
                        let res = client.get_sounds(&p).map_err(|e| e.to_string());
                        let act = match res { Ok(s) => ShopAction::Sounds(s), Err(e) => ShopAction::Error(e) };
                        if let Ok(s) = s.lock() { let _ = s.send(act); }
                    }
                });
            });
            cat_flow.insert(&btn, -1);
        }
        cat_scroll.set_child(Some(&cat_flow)); cat_box.append(&cat_scroll);
        container.append(&cat_box);

        let loading_page = GtkBox::new(Orientation::Vertical, 12);
        loading_page.set_valign(gtk4::Align::Center); loading_page.set_halign(gtk4::Align::Center);
        loading_page.append(&Label::new(Some("Загрузка...")));
        stack.add_named(&loading_page, Some("loading"));

        let listbox = ListBox::new();
        listbox.set_selection_mode(gtk4::SelectionMode::None);
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true); scrolled.set_child(Some(&listbox));
        stack.add_named(&scrolled, Some("results"));

        stack.add_named(&Label::new(Some("Выберите категорию")), Some("empty"));
        stack.set_visible_child_name("empty");
        container.append(&stack); container.append(&progress);

        let s_search = sender.clone(); let st_search = stack.clone(); let pr_search = progress.clone();
        search_btn.connect_clicked(glib::clone!(@weak search_entry => move |_| {
            let q = search_entry.text().to_string(); if q.is_empty() { return; }
            st_search.set_visible_child_name("loading"); pr_search.set_fraction(0.1);
            let s = s_search.clone();
            std::thread::spawn(move || {
                if let Ok(client) = MyInstantsClient::new() {
                    let res = client.search(&q).map_err(|e| e.to_string());
                    let act = match res { Ok(s) => ShopAction::Sounds(s), Err(e) => ShopAction::Error(e) };
                    if let Ok(s) = s.lock() { let _ = s.send(act); }
                }
            });
        }));

        let lb_ref = listbox.clone(); let st_ref = stack.clone(); let pr_ref = progress.clone();
        let state_ref = state.clone(); let sp_ref = soundpad.clone(); let s_dl = sender.clone();
        glib::idle_add_local(move || {
            while let Ok(action) = receiver.try_recv() {
                match action {
                    ShopAction::Sounds(sounds) => {
                        while let Some(c) = lb_ref.first_child() { lb_ref.remove(&c); }
                        for sound in sounds {
                            let row = GtkBox::new(Orientation::Horizontal, 8);
                            row.set_margin_start(8); row.set_margin_end(8);
                            let label = Label::new(Some(&sound.name));
                            label.set_hexpand(true); label.set_halign(gtk4::Align::Start);

                            // Кнопка превью
                            let preview_btn = Button::with_label("");
                            preview_btn.add_css_class("pill");
                            let preview_url = sound.url.clone();
                            preview_btn.connect_clicked(move |_| {
                                let url = preview_url.clone();
                                std::thread::spawn(move || {
                                    if let Ok(client) = MyInstantsClient::new() {
                                        let mp3 = if url.contains("/media/") { url.clone() } else { client.get_sound_mp3_url(&url).unwrap_or_default() };
                                        if mp3.is_empty() { return; }
                                        let full_url = if mp3.starts_with("http") { mp3 } else { format!("https://www.myinstants.com{}", mp3) };
                                        eprintln!("Preview: {}", full_url);
                                        let launch = format!("uridecodebin uri=\"{}\" ! audioconvert ! audioresample ! autoaudiosink", full_url);
                                        if let Ok(pipe) = gstreamer::parse::launch(&launch) {
                                            let pipe = pipe.downcast::<gstreamer::Pipeline>().unwrap_or_else(|_| gstreamer::Pipeline::new());
                                            if pipe.set_state(gstreamer::State::Playing).is_ok() {
                                                let bus = pipe.bus().unwrap();
                                                let msg = bus.timed_pop_filtered(
                                                    Some(gstreamer::ClockTime::from_seconds(30)),
                                                    &[gstreamer::MessageType::Eos, gstreamer::MessageType::Error],
                                                );
                                                if msg.is_none() {
                                                    eprintln!("Preview: таймаут");
                                                }
                                                let _ = pipe.set_state(gstreamer::State::Null);
                                            }
                                        }
                                    }
                                });
                            });

                            let btn = Button::with_label("Скачать"); btn.add_css_class("pill");
                            let u = sound.url.clone(); let n = sound.name.clone(); let s = s_dl.clone();
                            btn.connect_clicked(move |_| {
                                let u = u.clone(); let n = n.clone(); let s = s.clone();
                                std::thread::spawn(move || {
                                    if let Ok(client) = MyInstantsClient::new() {
                                        let mp3 = if u.contains("/media/") { u } else { client.get_sound_mp3_url(&u).unwrap_or_default() };
                                        if mp3.is_empty() { return; }
                                        let dest = dirs::data_dir().unwrap().join("soundpaad/downloads");
                                        let _ = std::fs::create_dir_all(&dest);
                                        let file = dest.join(format!("{}.mp3", n.replace("/", "_")));
                                        if client.download_sound(&mp3, &file).is_ok() {
                                            if let Ok(s) = s.lock() { let _ = s.send(ShopAction::Downloaded(n, file.to_string_lossy().to_string())); }
                                        }
                                    }
                                });
                            });
                            row.append(&label); row.append(&preview_btn); row.append(&btn); lb_ref.append(&row);
                        }
                        st_ref.set_visible_child_name("results"); pr_ref.set_fraction(0.0);
                    }
                    ShopAction::Downloaded(name, path) => {
                        let s = state_ref.borrow();
                        let _ = s.db.add_track(&crate::db::models::Track { id: None, name, file_path: path, category_id: None, volume: 1.0, source: "shop".to_string() });
                        drop(s); sp_ref.refresh();
                    }
                    ShopAction::Error(_) => { st_ref.set_visible_child_name("empty"); pr_ref.set_fraction(0.0); }
                }
            }
            glib::ControlFlow::Continue
        });
        Self { container, listbox, stack, progress, state, _soundpad: soundpad }
    }
    pub fn widget(&self) -> gtk4::Widget { self.container.clone().upcast() }
}
