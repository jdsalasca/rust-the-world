mod actor;
mod ai;
mod combat;
mod economy;
mod game;
mod hud;
mod log;
mod render;
mod world;

use actor::{BODY_RADIUS, Life, Role};
use game::{DEFAULT_SEED, Game, GameState};
use macroquad::prelude::*;
use world::{MAP_H, MAP_W, Map, TILE_SIZE};

fn window_conf() -> Conf {
    Conf {
        window_title: "rust-the-world".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn place_props(map: &Map) -> Vec<(render::Prop, Vec2)> {
    let town = map.town_spawn();
    let camp = map.camp_spawn();
    vec![
        (render::Prop::Well, town + vec2(-112., -64.)),
        (render::Prop::Tent, town + vec2(64., -112.)),
        (render::Prop::Tent, town + vec2(-144., 48.)),
        (render::Prop::Fire, town + vec2(-16., 40.)),
        (render::Prop::Crate, town + vec2(48., 96.)),
        (render::Prop::Crate, town + vec2(-72., -96.)),
        (render::Prop::Tent, camp + vec2(-16., -96.)),
        (render::Prop::Fire, camp + vec2(32., 16.)),
        (render::Prop::Crate, camp + vec2(-80., 32.)),
        (render::Prop::Cactus, vec2(46., 14.) * TILE_SIZE),
        (render::Prop::Cactus, vec2(18., 20.) * TILE_SIZE),
        (render::Prop::Cactus, vec2(28., 31.) * TILE_SIZE),
        (render::Prop::Cactus, vec2(50., 40.) * TILE_SIZE),
        (render::Prop::Cactus, vec2(16., 26.) * TILE_SIZE),
        (render::Prop::Rock, vec2(40., 12.) * TILE_SIZE),
        (render::Prop::Rock, vec2(22., 34.) * TILE_SIZE),
        (render::Prop::Rock, vec2(54., 20.) * TILE_SIZE),
    ]
}

fn make_camera(map: &Map, focus: Vec2) -> Camera2D {
    // zoom positivo: macroquad invierte Y solo, así que el mundo queda con Y hacia abajo.
    Camera2D {
        zoom: vec2(4.0 / screen_width(), 4.0 / screen_height()),
        target: map.clamp_camera(focus),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let art = render::Art::load();
    let mut game = Game::new(DEFAULT_SEED);
    let props = place_props(&game.map);

    let args: Vec<String> = std::env::args().collect();
    let shot = args
        .iter()
        .position(|argument| argument == "--shot")
        .and_then(|index| args.get(index + 1).cloned());
    if args.iter().any(|argument| argument == "--shop")
        && let Some(merchant) = game
            .actors
            .iter()
            .position(|body| body.role == Role::Merchant)
    {
        let position = game.actors[merchant].pos;
        game.actors[0].pos = position + vec2(44., 0.);
        game.open_shop();
    }
    if args.iter().any(|argument| argument == "--dead") {
        game.actors[0].life = Life::Dead;
        game.state = GameState::Dead;
    }
    let mut frame = 0;

    loop {
        let dt = get_frame_time();
        let camera = make_camera(&game.map, game.player().pos);
        let mouse_world = camera.screen_to_world(mouse_position().into());

        if is_key_down(KeyCode::Escape) && game.state != GameState::Shop {
            break;
        }

        match game.state {
            GameState::Playing => {
                if is_key_pressed(KeyCode::Space) {
                    game.paused = !game.paused;
                }
                if is_key_pressed(KeyCode::Tab) {
                    game.speed = if game.speed > 1.0 { 1.0 } else { 2.0 };
                }
                if is_key_pressed(KeyCode::E) {
                    game.open_shop();
                }
                if is_key_pressed(KeyCode::F) {
                    game.eat();
                }
                if is_key_pressed(KeyCode::G) {
                    game.bandage();
                }

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
                if dir != Vec2::ZERO && !game.paused && game.player().life == Life::Up {
                    let index = game.player;
                    let player = &mut game.actors[index];
                    let delta = dir.normalize() * player.speed() * dt * game.speed;
                    let next_x = vec2(player.pos.x + delta.x, player.pos.y);
                    if game.map.walkable(next_x, BODY_RADIUS) {
                        player.pos.x = next_x.x;
                    }
                    let next_y = vec2(player.pos.x, player.pos.y + delta.y);
                    if game.map.walkable(next_y, BODY_RADIUS) {
                        player.pos.y = next_y.y;
                    }
                    if delta.x != 0.0 {
                        player.facing_right = delta.x >= 0.0;
                    }
                    player.moving = true;
                }

                if is_mouse_button_pressed(MouseButton::Left) && !game.paused {
                    let aim = (mouse_world - game.player().pos).normalize_or_zero();
                    game.attack(aim);
                }
            }
            GameState::Shop => {
                if is_key_pressed(KeyCode::Key1) {
                    game.buy_food();
                }
                if is_key_pressed(KeyCode::Key2) {
                    game.buy_bandage();
                }
                if is_key_pressed(KeyCode::Key3) {
                    game.sell_spare();
                }
                if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Escape) {
                    game.close_shop();
                }
            }
            GameState::Dead => {
                if is_key_pressed(KeyCode::R) {
                    game.restart();
                }
            }
        }

        game.tick(dt);

        let player_pos = game.player().pos;
        let camera = make_camera(&game.map, player_pos);
        set_camera(&camera);

        clear_background(Color::from_rgba(24, 22, 20, 255));
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                render::draw_tile(
                    &art,
                    game.map.tile(x, y),
                    vec2(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                );
            }
        }
        for (prop, at) in &props {
            render::draw_prop(&art, *prop, *at);
        }
        for drop in &game.drops {
            render::draw_item(&art, drop.item, drop.pos);
        }
        for body in &game.actors {
            let anim = if body.attack_cd > 0.0 {
                render::AnimKind::Attack
            } else if body.moving {
                render::AnimKind::Walk
            } else {
                render::AnimKind::Idle
            };
            render::draw_actor(
                art.sheet(body.role),
                anim,
                body.anim_time,
                body.pos,
                body.facing_right,
            );
        }

        set_default_camera();
        draw_text(
            "rust-the-world",
            22.0,
            24.0,
            22.0,
            Color::from_rgba(235, 225, 200, 255),
        );
        hud::draw(&game);

        frame += 1;
        if frame == 3
            && let Some(path) = &shot
        {
            get_screen_data().export_png(path);
            break;
        }

        next_frame().await;
    }
}
