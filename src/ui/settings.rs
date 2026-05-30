use adw::prelude::*;
use gtk4::{Box as GtkBox, Button, DropDown, Label, Orientation, StringList, Switch};
use std::cell::RefCell;
use std::rc::Rc;
use crate::audio::virtual_mic::AudioServer;
use crate::AppState;

pub struct SettingsPage {
    container: gtk4::Box,
    state: Rc<RefCell<AppState>>,
}

impl SettingsPage {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let container = gtk4::Box::new(Orientation::Vertical, 12);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);

        let title = Label::new(Some("Настройки"));
        title.add_css_class("title-2");
        container.append(&title);

        let mic_section = adw::PreferencesGroup::new();
        mic_section.set_title("Виртуальный микрофон");

        let server_name = match state.borrow().virtual_mic.server() {
            AudioServer::PipeWire => "PipeWire",
            AudioServer::PulseAudio => "PulseAudio",
        };

        mic_section.add(&adw::ActionRow::builder()
            .title("Аудио сервер")
            .subtitle(server_name)
            .build());

        let status_row = adw::ActionRow::builder()
            .title("Статус")
            .subtitle(if state.borrow().virtual_mic.is_active() { "Активен" } else { "Неактивен" })
            .build();
        mic_section.add(&status_row);
        container.append(&mic_section);

        // Секция выбора реального микрофона (создаём ДО кнопок, чтобы ссылаться из колбеков)
        let mic_select_section = adw::PreferencesGroup::new();
        mic_select_section.set_title("Микрофон для наложения");

        let sources = crate::audio::virtual_mic::VirtualMic::list_sources()
            .unwrap_or_default();

        let source_names = StringList::new(&[]);
        let mut source_ids: Vec<String> = Vec::new();
        for src in &sources {
            let display = src
                .strip_prefix("alsa_input.")
                .or_else(|| src.strip_prefix("alsa_input_pci-"))
                .or_else(|| src.strip_prefix("pulse_input."))
                .unwrap_or(src)
                .replace('_', " ");
            source_names.append(&display);
            source_ids.push(src.clone());
        }
        if source_names.n_items() == 0 {
            source_names.append("(не найдены)");
            source_ids.push(String::new());
        }

        let mic_dropdown = DropDown::builder()
            .model(&source_names)
            .expression(&gtk4::PropertyExpression::new(
                gtk4::StringObject::static_type(),
                None::<&gtk4::Expression>,
                "string",
            ))
            .build();
        mic_dropdown.set_hexpand(true);

        let mic_select_row = adw::ActionRow::builder()
            .title("Реальный микрофон")
            .subtitle("Аудио из этого микрофона будет идти в виртуальный микрофон вместе со звуками")
            .build();
        mic_select_row.add_suffix(&mic_dropdown);
        mic_select_row.set_activatable_widget(Some(&mic_dropdown));
        mic_select_section.add(&mic_select_row);

        let mic_status_row = adw::ActionRow::builder()
            .title("Прохождение микрофона")
            .subtitle(if state.borrow().player.is_mic_passthrough_active() { "Активно" } else { "Неактивно" })
            .build();
        mic_select_section.add(&mic_status_row);
        container.append(&mic_select_section);

        // Кнопки создания/удаления виртуального микрофона
        let mic_btn_box = GtkBox::new(Orientation::Horizontal, 8);
        let create_btn = Button::with_label("Создать");
        create_btn.add_css_class("suggested-action");
        create_btn.add_css_class("pill");

        let destroy_btn = Button::with_label("Удалить");
        destroy_btn.add_css_class("destructive-action");
        destroy_btn.add_css_class("pill");

        let s_create = state.clone();
        let st_create = status_row.clone();
        let mic_st_create = mic_status_row.clone();
        create_btn.connect_clicked(move |_| {
            let mut s = s_create.borrow_mut();
            if let Ok(()) = s.virtual_mic.create() {
                let sink = s.virtual_mic.sink_name().to_string();
                s.player.set_virtual_mic_sink(&sink);
                s.player.start_mic_passthrough();
                st_create.set_subtitle("Активен");
                mic_st_create.set_subtitle(if s.player.is_mic_passthrough_active() { "Активно" } else { "Неактивно" });
            }
        });

        let s_destroy = state.clone();
        let st_destroy = status_row.clone();
        let mic_st_destroy = mic_status_row.clone();
        destroy_btn.connect_clicked(move |_| {
            let mut s = s_destroy.borrow_mut();
            s.player.stop_mic_passthrough();
            if let Ok(()) = s.virtual_mic.destroy() {
                st_destroy.set_subtitle("Неактивен");
                mic_st_destroy.set_subtitle("Неактивно");
            }
        });

        mic_btn_box.append(&create_btn);
        mic_btn_box.append(&destroy_btn);
        container.append(&mic_btn_box);

        let play_section = adw::PreferencesGroup::new();
        play_section.set_title("Воспроизведение");

        let spk_row = adw::ActionRow::builder()
            .title("Играть в наушниках/колонках")
            .subtitle("Звук слышен и у вас, и в микрофоне")
            .build();
        let spk_sw = Switch::new();
        spk_sw.set_active(state.borrow().player.play_to_speakers());
        spk_sw.set_valign(gtk4::Align::Center);
        spk_row.add_suffix(&spk_sw);
        spk_row.set_activatable_widget(Some(&spk_sw));
        play_section.add(&spk_row);

        let mic_row = adw::ActionRow::builder()
            .title("Только виртуальный микрофон")
            .subtitle("Звук слышен только в приложениях")
            .build();
        let mic_sw = Switch::new();
        mic_sw.set_active(!state.borrow().player.play_to_speakers());
        mic_sw.set_valign(gtk4::Align::Center);
        mic_row.add_suffix(&mic_sw);
        mic_row.set_activatable_widget(Some(&mic_sw));
        play_section.add(&mic_row);
        container.append(&play_section);

        let s_spk = state.clone();
        let mo_ref = mic_sw.clone();
        spk_sw.connect_active_notify(move |sw| {
            let active = sw.is_active();
            if active != !mo_ref.is_active() { mo_ref.set_active(!active); }
            s_spk.borrow_mut().player.set_play_to_speakers(active);
        });

        let s_mo = state.clone();
        let spk_ref = spk_sw.clone();
        mic_sw.connect_active_notify(move |sw| {
            let active = sw.is_active();
            if active != spk_ref.is_active() { spk_ref.set_active(!active); }
            s_mo.borrow_mut().player.set_play_to_speakers(!active);
        });

        let info = Label::new(Some("Выберите 'SoundPad_Mic' как микрофон в приложениях."));
        info.set_wrap(true);
        info.add_css_class("dim-label");
        container.append(&info);

        // Обработчик выбора микрофона
        let s_mic = state.clone();
        let mic_st_ref = mic_status_row.clone();
        mic_dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            let s = s_mic.borrow();
            if idx < source_ids.len() && !source_ids[idx].is_empty() {
                let src = source_ids[idx].clone();
                drop(s);
                let mut s = s_mic.borrow_mut();
                s.player.set_real_mic_source(&src);
                if s.player.is_mic_passthrough_active() {
                    mic_st_ref.set_subtitle("Активно");
                } else {
                    mic_st_ref.set_subtitle("Неактивно — создайте виртуальный микрофон");
                }
            }
        });

        Self { container, state }
    }
    pub fn widget(&self) -> gtk4::Widget { self.container.clone().upcast() }
}
