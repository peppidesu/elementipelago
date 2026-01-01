use std::default::Default;

use macroquad::{
    camera::{Camera2D, set_camera},
    color::{BLACK, GREEN, WHITE},
    math::vec2,
    miniquad::window::screen_size,
    text::{Font, TextParams, draw_text_ex},
    texture::get_screen_data,
    ui::{
        self, Skin, hash, root_ui,
        widgets::{self, Editbox, Group, InputText, Label},
    },
    window::{clear_background, next_frame, screen_height, screen_width},
};
use quad_storage::LocalStorage;

struct InputFields {
    url: String,
    slot: String,
    password: String,
}

pub async fn enter_login(storage: &mut LocalStorage, font: &Font) {
    storage.set("login_complete", "false");

    let url = storage.get("url").unwrap_or_else(|| {
        storage.set("url", "");
        "".to_string()
    });
    let slot = storage.get("slot").unwrap_or_else(|| {
        storage.set("slot", "");
        "".to_string()
    });
    let password = storage.get("password").unwrap_or_else(|| {
        storage.set("password", "");
        "".to_string()
    });

    let mut input_fields = InputFields {
        url,
        slot,
        password,
    };

    loop {
        if render_login(font, &mut input_fields).await {
            break;
        }

        next_frame().await
    }
}

async fn render_login(font: &Font, tmp: &mut InputFields) -> bool {
    let title_skin = {
        let label_style = root_ui()
            .style_builder()
            .with_font(&font)
            .unwrap()
            .text_color(BLACK)
            .font_size(48)
            .build();

        Skin {
            label_style,
            ..root_ui().default_skin()
        }
    };

    clear_background(WHITE);

    let screen_size = screen_size();
    let screen_size = vec2(screen_size.0, screen_size.1);

    let login_box_size = screen_size * 0.4;
    let login_box_offset = screen_size * 0.3;

    widgets::Group::new(hash!(), login_box_size)
        .position(login_box_offset)
        .ui(&mut *root_ui(), |ui| {
            ui.push_skin(&title_skin);
            ui.label(None, "Elementipelago");
            ui.pop_skin();
            Group::new(hash!(), login_box_size * vec2(1., 0.8))
                .position(login_box_size * vec2(0., 0.2))
                .ui(ui, |ui| {
                    ui.input_text(hash!(), "Server", &mut tmp.url);
                    ui.input_text(hash!(), "Slot", &mut tmp.slot);
                    ui.input_text(hash!(), "Password", &mut tmp.password);
                });
        });

    false
}
