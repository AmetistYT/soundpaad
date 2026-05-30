use adw::prelude::*;
use adw::ApplicationWindow;
use gtk4::Box as GtkBox;
use gtk4::{HeaderBar, Notebook, Orientation};
use gtk4::glib::translate::IntoGlib;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;

use super::binds::BindsPage;
use super::settings::SettingsPage;
use super::shop::ShopPage;
use super::soundpad::SoundpadPage;

pub struct AppUI {
    window: ApplicationWindow,
    _soundpad: Rc<SoundpadPage>,
    _shop_page: ShopPage,
    _settings_page: SettingsPage,
    _binds_page: Rc<BindsPage>,
}

impl AppUI {
    pub fn build(app: &adw::Application, state: Rc<RefCell<AppState>>) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("SoundPad")
            .default_width(900)
            .default_height(650)
            .build();

        let content = GtkBox::new(Orientation::Vertical, 0);

        let header = HeaderBar::new();
        content.append(&header);

        let notebook = Notebook::new();

        let binds_page = BindsPage::new(state.clone());
        let soundpad_page = SoundpadPage::new(state.clone(), binds_page.clone());
        let shop_page = ShopPage::new(state.clone(), soundpad_page.clone());
        let settings_page = SettingsPage::new(state.clone());

        notebook.append_page(&soundpad_page.widget(), Some(&gtk4::Label::new(Some("Саундпад"))));
        notebook.append_page(&shop_page.widget(), Some(&gtk4::Label::new(Some("Магазин"))));
        notebook.append_page(&binds_page.widget(), Some(&gtk4::Label::new(Some("Бинды"))));
        notebook.append_page(&settings_page.widget(), Some(&gtk4::Label::new(Some("Настройки"))));

        content.append(&notebook);
        window.set_content(Some(&content));

        // Обработчик горячих клавиш для биндов
        let state_key = state.clone();
        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(move |_controller, keyval, _keycode, modifier| {
            let key_code = keyval.to_unicode().map(|c| c as u32).unwrap_or(0);
            let mod_bits = modifier.bits() as u32;
            
            // Если unicode 0, пробуем использовать raw keyval
            let search_key = if key_code == 0 { keyval.into_glib() as u32 } else { key_code };
            
            let s = state_key.borrow();
            if let Ok(Some(bind)) = s.db.get_bind_by_key(search_key, mod_bits) {
                if let Ok(tracks) = s.db.get_tracks() {
                    if let Some(track) = tracks.iter().find(|t| t.id == Some(bind.track_id)) {
                        let track_id = track.id.unwrap_or(0);
                        let track_path = track.file_path.clone();
                        let track_volume = track.volume;
                        drop(s);
                        let mut s = state_key.borrow_mut();
                        if let Err(e) = s.player.play(track_id, &track_path, track_volume) {
                            eprintln!("Ошибка воспроизведения бинда: {}", e);
                        }
                        return gtk4::glib::Propagation::Stop;
                    }
                }
            }
            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(controller);

        window.show();

        Self {
            window,
            _soundpad: soundpad_page,
            _shop_page: shop_page,
            _settings_page: settings_page,
            _binds_page: binds_page,
        }
    }
}
