use adw::prelude::*;
use gtk4::{Button, FileDialog, FlowBox, FlowBoxChild, Label, Orientation, ScrolledWindow, gio};
use gtk4::gio::{Cancellable, SimpleAction, SimpleActionGroup};
use gtk4::glib::{self, translate::IntoGlib};
use std::cell::RefCell;
use std::rc::Rc;
use crate::AppState;
use super::binds::BindsPage;

pub struct SoundpadPage {
    container: gtk4::Box,
    flowbox: FlowBox,
    state: Rc<RefCell<AppState>>,
    binds_page: Rc<BindsPage>,
}

impl SoundpadPage {
    pub fn new(state: Rc<RefCell<AppState>>, binds_page: Rc<BindsPage>) -> Rc<Self> {
        let container = gtk4::Box::new(Orientation::Vertical, 8);
        container.set_margin_start(12); container.set_margin_end(12);
        container.set_margin_top(12); container.set_margin_bottom(12);

        let header_box = gtk4::Box::new(Orientation::Horizontal, 8);
        let title = Label::new(Some("Нажмите на звук для воспроизведения"));
        title.add_css_class("title-4"); title.set_hexpand(true); title.set_halign(gtk4::Align::Start);
        header_box.append(&title);

        let add_btn = Button::with_label("Добавить"); add_btn.add_css_class("pill");
        let refresh_btn = Button::with_label("Обновить"); refresh_btn.add_css_class("pill");
        let stop_btn = Button::with_label("Стоп"); stop_btn.add_css_class("pill"); stop_btn.add_css_class("destructive-action");

        header_box.append(&add_btn); header_box.append(&refresh_btn); header_box.append(&stop_btn);
        container.append(&header_box);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true); scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

        let flowbox = FlowBox::new();
        flowbox.set_selection_mode(gtk4::SelectionMode::None);
        flowbox.set_column_spacing(10); flowbox.set_row_spacing(10);
        flowbox.set_homogeneous(true); flowbox.set_min_children_per_line(4); flowbox.set_max_children_per_line(10);
        flowbox.set_valign(gtk4::Align::Start);

        scrolled.set_child(Some(&flowbox)); container.append(&scrolled);

        let page = Rc::new(Self { container, flowbox, state, binds_page });

        let audio_filter = gtk4::FileFilter::new();
        audio_filter.set_name(Some("Аудиофайлы")); audio_filter.add_mime_type("audio/*");
        audio_filter.add_suffix("mp3"); audio_filter.add_suffix("wav"); audio_filter.add_suffix("ogg"); audio_filter.add_suffix("flac");
        let filters_store = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
        filters_store.append(&audio_filter);

        let p_add = page.clone();
        add_btn.connect_clicked(move |_| {
            let dialog = FileDialog::builder().title("Выберите аудиофайл").filters(&filters_store).build();
            let p = p_add.clone();
            dialog.open(None::<&gtk4::Window>, Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
                        let file_path = path.to_string_lossy().to_string();
                        let s = p.state.borrow();
                        if let Ok(_) = s.db.add_track(&crate::db::models::Track { id: None, name, file_path, category_id: None, volume: 1.0, source: "local".to_string() }) {
                            drop(s); p.refresh();
                        }
                    }
                }
            });
        });

        let p_refresh = page.clone();
        refresh_btn.connect_clicked(move |_| { p_refresh.refresh(); });

        let s_stop = page.state.clone();
        stop_btn.connect_clicked(move |_| { s_stop.borrow_mut().player.stop_all(); });

        page.refresh();
        page
    }

    pub fn refresh(self: &Rc<Self>) {
        while let Some(child) = self.flowbox.first_child() { self.flowbox.remove(&child); }
        let tracks = self.state.borrow().db.get_tracks().unwrap_or_default();
        if tracks.is_empty() {
            let empty = Label::new(Some("Нет звуков. Добавьте файлы или скачайте из магазина."));
            empty.add_css_class("dim-label"); empty.set_margin_top(20);
            self.flowbox.insert(&empty, -1);
            return;
        }
        for track in &tracks {
            let track_id = track.id.unwrap_or(0); let track_name = track.name.clone();
            let track_path = track.file_path.clone(); let track_volume = track.volume;
            
            let btn = Button::with_label(&track_name);
            btn.add_css_class("pill"); btn.add_css_class("suggested-action");
            btn.set_size_request(100, 40); btn.set_valign(gtk4::Align::Center);

            let s_play = self.state.clone();
            btn.connect_clicked(move |_| {
                let mut s = s_play.borrow_mut();
                let _ = s.player.play(track_id, &track_path, track_volume);
            });

            let actions = SimpleActionGroup::new();
            let s_bind = self.state.clone();
            let b_page = self.binds_page.clone();
            let bind_action = SimpleAction::new("bind", None);
            let tname_bind = track_name.clone();
            bind_action.connect_activate(move |_action, _param| { show_bind_dialog(&s_bind, track_id, &tname_bind, &b_page); });
            actions.add_action(&bind_action);

            let s_del = self.state.clone(); let p_refresh = self.clone();
            let del_action = SimpleAction::new("delete", None);
            del_action.connect_activate(move |_action, _param| {
                let s = s_del.borrow();
                if let Ok(()) = s.db.delete_track(track_id) { let _ = s.db.delete_bind_by_track(track_id); drop(s); p_refresh.refresh(); p_refresh.binds_page.refresh(); }
            });
            actions.add_action(&del_action);

            btn.insert_action_group("track", Some(&actions));
            let gesture = gtk4::GestureClick::new();
            gesture.set_button(gtk4::gdk::BUTTON_SECONDARY);
            let tname_menu = track_name.clone();
            gesture.connect_pressed(move |gesture, _n, _x, _y| {
                if gesture.current_button() != gtk4::gdk::BUTTON_SECONDARY {
                    gesture.set_state(gtk4::EventSequenceState::Denied);
                    return;
                }
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                let menu = gio::Menu::new();
                menu.append(Some(&format!("Назначить хоткей: {}", tname_menu)), Some("track.bind"));
                menu.append(Some("Удалить"), Some("track.delete"));
                let popover = gtk4::PopoverMenu::from_model(Some(&menu)); popover.set_has_arrow(false);
                let btn_widget = gesture.widget().unwrap(); popover.set_parent(&btn_widget); popover.show();
            });
            btn.add_controller(gesture);
            self.flowbox.insert(&btn, -1);
        }
    }
    pub fn widget(&self) -> gtk4::Widget { self.container.clone().upcast() }
}

fn show_bind_dialog(state: &Rc<RefCell<AppState>>, track_id: i64, track_name: &str, binds_page: &Rc<BindsPage>) {
    let dialog = gtk4::MessageDialog::new(None::<&gtk4::Window>, gtk4::DialogFlags::MODAL, gtk4::MessageType::Question, gtk4::ButtonsType::None, &format!("Назначить хоткей для «{}»\nНажмите комбинацию клавиш...", track_name));
    dialog.add_button("Отмена", gtk4::ResponseType::Cancel); dialog.add_button("Сохранить", gtk4::ResponseType::Accept);
    let keyval_label = Label::new(Some("(нажмите клавишу)")); keyval_label.add_css_class("title-4");
    dialog.content_area().append(&keyval_label);
    let captured_key: Rc<RefCell<Option<(u32, u32)>>> = Rc::new(RefCell::new(None));
    let controller = gtk4::EventControllerKey::new();
    let captured_clone = captured_key.clone(); let label_clone = keyval_label.clone();
    controller.connect_key_pressed(move |_ctrl, keyval, _keycode, modifier| {
        let mod_bits = modifier.bits() as u32; let mut parts = Vec::new();
        if mod_bits & gtk4::gdk::ModifierType::CONTROL_MASK.bits() != 0 { parts.push("Ctrl"); }
        if mod_bits & gtk4::gdk::ModifierType::SHIFT_MASK.bits() != 0 { parts.push("Shift"); }
        if mod_bits & gtk4::gdk::ModifierType::ALT_MASK.bits() != 0 { parts.push("Alt"); }
        if mod_bits & gtk4::gdk::ModifierType::SUPER_MASK.bits() != 0 { parts.push("Super"); }
        let key_name = if let Some(c) = gtk4::gdk::Key::to_unicode(&keyval) { c.to_uppercase().to_string() } else { gtk4::gdk::Key::name(&keyval).map(|s| s.to_string()).unwrap_or_else(|| "?".to_string()) };
        parts.push(&key_name); label_clone.set_label(&parts.join("+"));
        *captured_clone.borrow_mut() = Some((keyval.into_glib() as u32, mod_bits));
        glib::Propagation::Stop
    });
    dialog.add_controller(controller);
    let s_save = state.clone(); let captured_save = captured_key.clone();
    let b_page = binds_page.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Accept {
            if let Some((kv, mods)) = *captured_save.borrow() {
                let s = s_save.borrow();
                if let Ok(Some(ex)) = s.db.get_bind_by_key(kv, mods) { let _ = s.db.delete_bind(ex.id.unwrap_or(0)); }
                let _ = s.db.add_bind(&crate::db::models::Bind { id: None, track_id, keyval: kv, modifiers: mods });
                drop(s);
                b_page.refresh();
            }
        }
        dialog.close();
    });
    dialog.show();
}
