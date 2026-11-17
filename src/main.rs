mod actor;
mod world;

use actor::{Actor, BODY_RADIUS, Role};
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

fn role_color(role: Role) -> Color {
    match role {
        Role::Player => Color::from_rgba(232, 224, 200, 255),
        Role::Guard => Color::from_rgba(74, 111, 165, 255),
        Role::Merchant => Color::from_rgba(95, 138, 74, 255),
        Role::Civilian => Color::from_rgba(201, 179, 126, 255),
        Role::Bandit => Color::from_rgba(165, 59, 59, 255),
        Role::Dog => Color::from_rgba(107, 74, 47, 255),
    }
}

/// Primer punto caminable alrededor de `at`, para no nacer dentro de una roca.
fn free_spot(map: &Map, at: Vec2) -> Vec2 {
    let offsets = [vec2(0., 0.), vec2(48., 0.), vec2(-48., 0.), vec2(0., 48.), vec2(0., -48.), vec2(64., 64.), vec2(-64., -64.)];
    offsets
        .iter()
        .map(|offset| at + *offset)
        .find(|point| map.walkable(*point, BODY_RADIUS + 2.0))
        .unwrap_or(at)
}

fn spawn_actors(map: &Map) -> Vec<Actor> {
    let town = map.town_spawn();
    let camp = map.camp_spawn();
    let mut actors = Vec::new();
    let mut add = |pos: Vec2, role: Role| {
        let id = actors.len();
        actors.push(Actor::new(id, free_spot(map, pos), role));
    };

    add(town, Role::Player);
    add(town + vec2(64., -48.), Role::Merchant);
    add(town + vec2(-96., 32.), Role::Guard);
    add(town + vec2(96., 48.), Role::Guard);
    add(town + vec2(-48., 80.), Role::Civilian);
    add(town + vec2(48., 96.), Role::Civilian);

    add(camp + vec2(-48., -32.), Role::Bandit);
    add(camp + vec2(32., -48.), Role::Bandit);
    add(camp + vec2(-32., 48.), Role::Bandit);
    add(camp + vec2(48., 32.), Role::Bandit);

    add(vec2(46., 20.) * TILE_SIZE, Role::Dog);
    add(vec2(48., 22.) * TILE_SIZE, Role::Dog);
    add(vec2(44., 22.) * TILE_SIZE, Role::Dog);

    actors
}

#[macroquad::main(window_conf)]
async fn main() {
    let map = Map::generate(7);
    let mut actors = spawn_actors(&map);
    let mut facing_right = true;
    let shot = std::env::args().nth(1).filter(|flag| flag == "--shot");
    let shot_path = std::env::args().nth(2);
    let mut frame = 0;

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }

        let dt = get_frame_time();
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dir.x += 1.0;
        }
        if dir != Vec2::ZERO {
            let player = &mut actors[0];
            let delta = dir.normalize() * player.speed() * dt;
            let next_x = vec2(player.pos.x + delta.x, player.pos.y);
            if map.walkable(next_x, BODY_RADIUS) {
                player.pos.x = next_x.x;
            }
            let next_y = vec2(player.pos.x, player.pos.y + delta.y);
            if map.walkable(next_y, BODY_RADIUS) {
                player.pos.y = next_y.y;
            }
            if delta.x != 0.0 {
                facing_right = delta.x > 0.0;
            }
        }
        let player_pos = actors[0].pos;

        let mut camera = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            screen_width() / 2.0,
            screen_height() / 2.0,
        ));
        camera.target = map.clamp_camera(player_pos);
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
        for body in &actors {
            draw_circle(body.pos.x, body.pos.y, BODY_RADIUS, role_color(body.role));
        }
        draw_rectangle(
            player_pos.x - 2.0,
            player_pos.y - 6.0,
            3.0,
            3.0,
            if facing_right { YELLOW } else { BLACK },
        );

        set_default_camera();
        draw_text("rust-the-world", 20.0, 40.0, 32.0, WHITE);
        draw_text("WASD mover · Esc salir", 20.0, 70.0, 22.0, GRAY);

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
