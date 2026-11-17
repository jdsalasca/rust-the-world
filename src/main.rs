mod actor;
mod ai;
mod combat;
mod economy;
mod log;
mod render;
mod world;

use actor::{Actor, BODY_RADIUS, Faction, Life, Role};
use economy::Drop;
use log::EventLog;
use macroquad::prelude::*;
use world::{MAP_H, MAP_W, Map, Rng, TILE_SIZE};

fn window_conf() -> Conf {
    Conf {
        window_title: "rust-the-world".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

/// Primer punto caminable alrededor de `at`, para no nacer dentro de una roca.
fn free_spot(map: &Map, at: Vec2) -> Vec2 {
    let offsets = [
        vec2(0., 0.),
        vec2(48., 0.),
        vec2(-48., 0.),
        vec2(0., 48.),
        vec2(0., -48.),
        vec2(64., 64.),
        vec2(-64., -64.),
    ];
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
    let mut camera = Camera2D::default();
    camera.zoom = vec2(4.0 / screen_width(), 4.0 / screen_height());
    camera.target = map.clamp_camera(focus);
    camera
}

#[macroquad::main(window_conf)]
async fn main() {
    let map = Map::generate(7);
    let art = render::Art::load();
    let props = place_props(&map);
    let mut actors = spawn_actors(&map);
    let mut drops: Vec<Drop> = Vec::new();
    let mut log = EventLog::new();
    let mut rng = Rng::new(11);
        let shot = std::env::args().nth(1).filter(|flag| flag == "--shot");
    let shot_path = std::env::args().nth(2);
    let mut frame = 0;

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }

        let dt = get_frame_time();
        let camera = make_camera(&map, actors[0].pos);
        let mouse_world = camera.screen_to_world(mouse_position().into());

        for body in actors.iter_mut() {
            body.moving = false;
            body.anim_time += dt;
            for event in body.tick(dt) {
                log.push(event);
            }
            body.attack_cd = (body.attack_cd - dt).max(0.0);
        }
        for event in ai::tick(&mut actors, &map, dt, &mut rng) {
            log.push(event);
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
        if dir != Vec2::ZERO && actors[0].life == Life::Up {
            actors[0].moving = true;
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
                player.facing_right = delta.x >= 0.0;
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) && actors[0].attack_cd <= 0.0 && actors[0].life == Life::Up {
            let aim = (mouse_world - actors[0].pos).normalize_or_zero();
            let mut best: Option<(usize, f32)> = None;
            for (index, other) in actors.iter().enumerate().skip(1) {
                if other.life == Life::Dead {
                    continue;
                }
                let offset = other.pos - actors[0].pos;
                let distance = offset.length();
                if distance > actors[0].weapon.reach() {
                    continue;
                }
                if aim != Vec2::ZERO && !combat::in_arc(&actors[0], other, aim) {
                    continue;
                }
                if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                    best = Some((index, distance));
                }
            }
            if let Some((index, _)) = best {
                let cooldown = actors[0].weapon.cooldown();
                let hit = {
                    let (before, after) = actors.split_at_mut(index);
                    combat::resolve_hit(&before[0], &mut after[0], &mut rng)
                };
                actors[0].attack_cd = cooldown;
                if let Some(hit) = hit {
                    log.push(format!(
                        "Golpeas a {} ({})",
                        actors[index].name(),
                        hit.limb.name()
                    ));
                    actors[index].hostile_to_player = true;
                    actors[index].target = Some(0);
                    actors[index].think = 0.5;
                    if actors[index].faction == Faction::Town {
                        for body in actors.iter_mut() {
                            if body.faction == Faction::Town {
                                body.hostile_to_player = true;
                            }
                        }
                    }
                }
            }
        }

        for body in actors.iter_mut() {
            if body.life == Life::Dead && !body.death_handled {
                log.push(format!("{} ha muerto", body.name()));
                drops.extend(economy::drop_loot(body));
            }
        }

        for event in economy::pick_up(&mut actors[0], &mut drops) {
            log.push(event);
        }
        let player_pos = actors[0].pos;

        let camera = make_camera(&map, player_pos);
        set_camera(&camera);

        clear_background(Color::from_rgba(24, 22, 20, 255));
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                render::draw_tile(
                    &art,
                    map.tile(x, y),
                    vec2(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                );
            }
        }
        for (prop, at) in &props {
            render::draw_prop(&art, *prop, *at);
        }
        for drop in &drops {
            render::draw_item(&art, drop.item, drop.pos);
        }
        for body in &actors {
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
        draw_text("rust-the-world", 20.0, 40.0, 32.0, WHITE);
        draw_text(
            "WASD mover · clic atacar · Esc salir",
            20.0,
            70.0,
            22.0,
            GRAY,
        );

        let lines: Vec<&String> = log.lines().iter().collect();
        for (index, line) in lines.iter().enumerate() {
            let y = screen_height() - 24.0 - (lines.len() - 1 - index) as f32 * 22.0;
            draw_text(line, 20.0, y, 20.0, LIGHTGRAY);
        }

        if actors[0].life == Life::Dead {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::from_rgba(0, 0, 0, 170),
            );
            draw_text("HAS MUERTO", screen_width() / 2.0 - 110.0, screen_height() / 2.0, 42.0, WHITE);
        }

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




