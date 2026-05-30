use adw::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Label, ListBox, Orientation, ScrolledWindow,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::db::models::Bind;
use crate::AppState;

pub struct BindsPage {
    container: gtk4::Box,
    listbox: ListBox,
    state: Rc<RefCell<AppState>>,
}

impl BindsPage {
    pub fn new(state: Rc<RefCell<AppState>>) -> Rc<Self> {
        let container = gtk4::Box::new(Orientation::Vertical, 8);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);

        let title = Label::new(Some("Нажмите кнопку и клавишу для назначения бинда"));
        title.add_css_class("title-4");
        container.append(&title);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        let listbox = ListBox::new();
        listbox.set_selection_mode(gtk4::SelectionMode::None);
        scrolled.set_child(Some(&listbox));
        container.append(&scrolled);

        let page = Rc::new(Self {
            container,
            listbox,
            state,
        });
        page.refresh();
        page
    }

    pub fn refresh(self: &Rc<Self>) {
        while let Some(child) = self.listbox.first_child() {
            self.listbox.remove(&child);
        }

        let s = self.state.borrow();
        let binds_list = s.db.get_binds().unwrap_or_default();

        for bind in &binds_list {
            let track_name = s.db.get_tracks()
                .ok()
                .and_then(|tracks| tracks.iter().find(|t| t.id == Some(bind.track_id)).cloned())
                .map(|t| t.name)
                .unwrap_or_else(|| "Неизвестный трек".to_string());

            let key_str = format_key(bind.keyval, bind.modifiers);
            let row_box = GtkBox::new(Orientation::Horizontal, 12);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);

            let label = Label::new(Some(&format!("{} → {}", key_str, track_name)));
            label.set_hexpand(true);
            label.set_halign(gtk4::Align::Start);

            let delete_btn = Button::with_label("Удалить");
            delete_btn.add_css_class("destructive-action");
            delete_btn.add_css_class("pill");

            let bind_id = bind.id.unwrap_or(0);
            let page = self.clone();
            let state_del = self.state.clone();
            delete_btn.connect_clicked(move |_| {
                let s = state_del.borrow();
                let _ = s.db.delete_bind(bind_id);
                drop(s);
                page.refresh();
            });

            row_box.append(&label);
            row_box.append(&delete_btn);
            self.listbox.append(&row_box);
        }

        if binds_list.is_empty() {
            let empty = Label::new(Some("Нет биндов. Добавьте звуки и назначьте хоткеи."));
            self.listbox.append(&empty);
        }
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }
}

fn format_key(keyval: u32, modifiers: u32) -> String {
    let mut parts = Vec::new();

    if modifiers & gtk4::gdk::ModifierType::CONTROL_MASK.bits() != 0 {
        parts.push("Ctrl");
    }
    if modifiers & gtk4::gdk::ModifierType::SHIFT_MASK.bits() != 0 {
        parts.push("Shift");
    }
    if modifiers & gtk4::gdk::ModifierType::ALT_MASK.bits() != 0 {
        parts.push("Alt");
    }
    if modifiers & gtk4::gdk::ModifierType::SUPER_MASK.bits() != 0 {
        parts.push("Super");
    }

    let key_name = match keyval {
        32 => "Space".to_string(),
        65288 => "BackSpace".to_string(),
        65289 => "Tab".to_string(),
        65293 => "Return".to_string(),
        65307 => "Escape".to_string(),
        65361 => "Left".to_string(),
        65362 => "Up".to_string(),
        65363 => "Right".to_string(),
        65364 => "Down".to_string(),
        kv if kv >= 97 && kv <= 122 => ((kv as u8) as char).to_uppercase().to_string(),
        kv if kv >= 48 && kv <= 57 => (kv as u8 as char).to_string(),
        kv => format!("Key{}", kv),
    };

    parts.push(&key_name);
    parts.join("+")
}
