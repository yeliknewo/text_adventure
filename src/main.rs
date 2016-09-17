
extern crate yaml_rust;
#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};

mod state;

use state::Game;

widget_ids!(
    struct Ids {
        canvas,
        text_edit
    }
);


fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, out_text: &mut String, in_text: &mut String) {
    use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

    for out_edit in widget::TextEdit::new(out_text)
        .color(color::LIGHT_BLUE)
        .padded_wh_of(ids.canvas, 20.0)
        .mid_top_of(ids.canvas)
        .align_text_x_middle()
        .line_spacing(2.5)
        .set(ids.text_edit, ui) {
        *out_text = out_edit;
    }

    for in_edit in widget::TextEdit::new(in_text)
        .color(color::BLACK)
        .padded_wh_of(ids.canvas, 20.0)
        .mid_bottom_of(ids.canvas)
        .align_text_x_middle()
        .line_spacing(2.5)
        .set(ids.text_edit, ui) {
        *in_text = in_edit;
    }
}

fn main() {
    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 300;

    let mut window: PistonWindow = WindowSettings::new("Text Adventure", [WIDTH, HEIGHT])
        .opengl(piston_window::OpenGL::V3_2)
        .vsync(true)
        .samples(4)
        .exit_on_esc(true)
        .build()
        .unwrap();

    window.set_ups(60);

    let mut ui = conrod::UiBuilder::new().build();

    let ids = Ids::new(ui.widget_id_generator());

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let mut text_texture_cache = conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    let image_map = conrod::image::Map::new();

    let mut out_text = "".to_owned();
    let mut in_text = "".to_owned();

    let mut game = Game::new();

    while let Some(event) = window.next() {
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let in_text = &mut in_text;
            let mut clear = false;
            for c in in_text.chars() {
                if c == '\n' {
                    game.process(&in_text, &mut out_text);
                    clear = true;
                    break;
                }
            }
            if clear {
                in_text.clear();
            }

            let out_text = &mut out_text;
            set_ui(ui.set_widgets(), &ids, in_text, out_text);
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(image: &T) -> &T { image };
                conrod::backend::piston_window::draw(
                    c,
                    g,
                    primitives,
                    &mut text_texture_cache,
                    &image_map,
                    texture_from_image
                );
            }
        });
    }

    // let story = open_story(buffer).unwrap();
    //
    // let mut node_opt = Some(story);
    //
    // while node_opt.is_some() {
    //     let mut node = node_opt.take().unwrap();
    //     println!("{:?}", node.name);
    //     for choice in node.choices.iter() {
    //         println!("{:?}", choice.name);
    //     }
    // }
}
