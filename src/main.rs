use macroquad::{
    miniquad::window::screen_size,
    prelude::*,
    ui::{Skin, hash, root_ui, widgets},
};

mod login;

#[macroquad::main("Elementipelago")]
async fn main() {
    let font = load_ttf_font("assets/fuzzybubbles-regular.ttf")
        .await
        .unwrap();
    let skin = {
        let label_style = root_ui()
            .style_builder()
            .with_font(&font)
            .unwrap()
            .text_color(BLACK)
            .font_size(24)
            .margin(RectOffset::new(0., 0., 5., 5.))
            .build();

        let group_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../assets/ui-atlas.png"),
                    Some(ImageFormat::Png),
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(0., 0., 0., 0.))
            .build();

        let button_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../assets/ui-atlas.png"),
                    Some(ImageFormat::Png),
                )
                .unwrap()
                .sub_image(Rect::new(0., 0., 32., 32.)),
            )
            .background_margin(RectOffset::new(0., 0., 0., 0.))
            .with_font(&font)
            .unwrap()
            .text_color(BLACK)
            .font_size(20)
            .build();

        let editbox_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(0., 0., 0., 0.))
            .margin(RectOffset::new(0., 0., 5., 5.))
            .color(GREEN)
            .with_font(&font)
            .unwrap()
            .text_color(BLACK)
            .color_selected(PURPLE)
            .font_size(24)
            .build();

        Skin {
            label_style,
            button_style,
            group_style,
            editbox_style,
            ..root_ui().default_skin()
        }
    };

    root_ui().push_skin(&skin);
    loop {
        clear_background(WHITE);

        let mut storage = quad_storage::STORAGE.lock().unwrap();

        login::enter_login(&mut storage, &font).await;

        next_frame().await
    }
}
