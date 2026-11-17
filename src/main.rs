mod world;

use macroquad::prelude::*;
use world::{MAP_H, MAP_W, Map, TILE_SIZE, Tile};

fn window_conf() -> Conf {
    Conf {
        window_title: "rust-the-world".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Sand => Color::from_rgba(194, 161, 90, 255),
        Tile::Dirt => Color::from_rgba(138, 111, 71, 255),
        Tile::Road => Color::from_rgba(176, 141, 90, 255),
        Tile::Rock => Color::from_rgba(74, 74, 85, 255),
        Tile::Water => Color::from_rgba(59, 110, 165, 255),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let map = Map::generate(7);
    let focus = map.town_spawn();
    let shot = std::env::args().nth(1).filter(|flag| flag == "--shot");
    let shot_path = std::env::args().nth(2);
    let mut frame = 0;

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }

        let mut camera = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            screen_width() / 2.0,
            screen_height() / 2.0,
        ));
        camera.target = map.clamp_camera(focus);
        set_camera(&camera);

        clear_background(Color::from_rgba(24, 22, 20, 255));
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    tile_color(map.tile(x, y)),
                );
            }
        }
        draw_circle(focus.x, focus.y, 10.0, WHITE);

        set_default_camera();
        draw_text("rust-the-world", 20.0, 40.0, 32.0, WHITE);
        draw_text("Esc: salir", 20.0, 70.0, 22.0, GRAY);

        frame += 1;
        if shot.is_some() && frame == 3 {
            if let Some(path) = &shot_path {
                get_screen_data().export_png(path);
            }
            break;
        }

        next_frame().await;
    }
}
