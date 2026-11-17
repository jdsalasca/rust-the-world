use crate::actor::{Actor, AiState, BODY_RADIUS, Faction, Life, Role};
use crate::combat::resolve_hit;
use crate::world::{Map, Rng, TILE_SIZE};
use macroquad::prelude::*;

pub const DETECT: f32 = 220.0;
pub const DOG_DETECT: f32 = 160.0;
pub const FLEE_RANGE: f32 = 150.0;
pub const FLEE_SECS: f32 = 2.0;
pub const GIVE_UP_SECS: f32 = 2.0;

pub fn hostile(a: &Actor, b: &Actor) -> bool {
    if a.id == b.id || a.life == Life::Dead || b.life == Life::Dead {
        return false;
    }
    if a.role == Role::Player && b.hostile_to_player {
        return true;
    }
    if b.role == Role::Player && a.hostile_to_player {
        return true;
    }
    match (a.faction, b.faction) {
        (Faction::Bandit, Faction::Bandit) | (Faction::Wildlife, Faction::Wildlife) => false,
        (Faction::Bandit, _) | (_, Faction::Bandit) => true,
        (Faction::Wildlife, _) | (_, Faction::Wildlife) => true,
        _ => false,
    }
}

fn cell(pos: Vec2) -> IVec2 {
    ivec2(
        (pos.x / TILE_SIZE).floor() as i32,
        (pos.y / TILE_SIZE).floor() as i32,
    )
}

fn nearest_hostile(index: usize, actors: &[Actor], range: f32) -> Option<usize> {
    let actor = &actors[index];
    let mut best: Option<(usize, f32)> = None;
    for (other_index, other) in actors.iter().enumerate() {
        if other.life != Life::Up || !hostile(actor, other) {
            continue;
        }
        let distance = (other.pos - actor.pos).length();
        if distance > range {
            continue;
        }
        if best.is_none_or(|(_, best_distance)| distance < best_distance) {
            best = Some((other_index, distance));
        }
    }
    best.map(|(index, _)| index)
}

pub fn pick_target(index: usize, actors: &[Actor]) -> Option<usize> {
    let actor = &actors[index];
    if actor.life != Life::Up {
        return None;
    }
    if matches!(actor.role, Role::Player | Role::Civilian | Role::Merchant) {
        return None;
    }
    let range = if actor.role == Role::Dog { DOG_DETECT } else { DETECT };
    nearest_hostile(index, actors, range)
}

fn pair_mut(actors: &mut [Actor], i: usize, j: usize) -> (&mut Actor, &mut Actor) {
    debug_assert_ne!(i, j);
    if i < j {
        let (left, right) = actors.split_at_mut(j);
        (&mut left[i], &mut right[0])
    } else {
        let (left, right) = actors.split_at_mut(i);
        (&mut right[0], &mut left[j])
    }
}

fn move_along(actor: &mut Actor, dt: f32) {
    let mut budget = actor.speed() * dt;
    while budget > 0.0 {
        let Some(next) = actor.path.first().copied() else {
            break;
        };
        actor.moving = true;        let target = vec2(
            next.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            next.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        );
        let delta = target - actor.pos;
        let distance = delta.length();
        if distance > budget {
            actor.pos += delta / distance * budget;
            actor.facing_right = delta.x >= 0.0;
            break;
        }
        actor.pos = target;
        actor.path.remove(0);
        budget -= distance;
    }
}

pub fn tick(actors: &mut Vec<Actor>, map: &Map, dt: f32, rng: &mut Rng) -> Vec<String> {    let mut events = Vec::new();
    let count = actors.len();

    for actor in actors.iter_mut() {
        if actor.role == Role::Player || actor.life != Life::Up {
            continue;
        }
        actor.repath = (actor.repath - dt).max(0.0);
        actor.think = (actor.think - dt).max(0.0);
        actor.flee_timer = (actor.flee_timer - dt).max(0.0);
        actor.give_up = (actor.give_up - dt).max(0.0);
    }

    for index in 0..count {
        if actors[index].role == Role::Player || actors[index].life != Life::Up {
            continue;
        }
        if matches!(actors[index].role, Role::Civilian | Role::Merchant) {
            actors[index].target = None;
            if nearest_hostile(index, actors, FLEE_RANGE).is_some() {
                actors[index].ai = AiState::Flee;
                actors[index].flee_timer = FLEE_SECS;
                actors[index].path.clear();
            } else if actors[index].flee_timer > 0.0 {
                actors[index].ai = AiState::Flee;
            } else {
                actors[index].ai = AiState::Idle;
            }
            continue;
        }

        if actors[index].think <= 0.0 {
            actors[index].target = pick_target(index, actors);
            actors[index].think = 0.5;
        }

        let Some(target) = actors[index].target else {
            if actors[index].give_up > 0.0 {
                actors[index].ai = AiState::Idle;
                actors[index].path.clear();
                continue;
            }
            actors[index].ai = AiState::Patrol;
            if actors[index].repath <= 0.0 && actors[index].path.is_empty() {
                let offset = vec2(rng.range(-96, 96) as f32, rng.range(-96, 96) as f32);
                let home = actors[index].home;
                actors[index].path =
                    map.find_path(cell(actors[index].pos), cell(home + offset)).unwrap_or_default();
                actors[index].repath = 3.0;
            }
            continue;
        };

        let distance = (actors[target].pos - actors[index].pos).length();
        if distance <= actors[index].weapon.reach() - 2.0 {
            actors[index].ai = AiState::Attack;
            actors[index].path.clear();
            continue;
        }

        actors[index].ai = AiState::Chase;
        if actors[index].repath <= 0.0 {
            actors[index].path =
                map.find_path(cell(actors[index].pos), cell(actors[target].pos)).unwrap_or_default();
            actors[index].repath = 0.5;
            if actors[index].path.is_empty() {
                actors[index].target = None;
                actors[index].think = 0.5;
                actors[index].give_up = GIVE_UP_SECS;
                actors[index].ai = AiState::Idle;
            }
        }
    }

    for index in 0..count {
        if actors[index].role == Role::Player || actors[index].life != Life::Up {
            continue;
        }
        match actors[index].ai {
            AiState::Chase | AiState::Patrol => move_along(&mut actors[index], dt),
            AiState::Flee => {
                if let Some(threat) = nearest_hostile(index, actors, 400.0) {
                    let away = actors[index].pos - actors[threat].pos;
                    let direction = away.normalize_or_zero();
                    let next = actors[index].pos + direction * actors[index].speed() * dt;
                    if map.walkable(next, BODY_RADIUS) {
                        actors[index].pos = next;
                        actors[index].facing_right = direction.x >= 0.0;
                        actors[index].moving = true;
                    }
                }
            }
            _ => {}
        }
    }

    for index in 0..count {
        if actors[index].life != Life::Up || actors[index].ai != AiState::Attack {
            continue;
        }
        let Some(target) = actors[index].target else {
            continue;
        };
        if actors[index].attack_cd > 0.0 {
            continue;
        }
        let distance = (actors[target].pos - actors[index].pos).length();
        if distance > actors[index].weapon.reach() {
            continue;
        }
        let cooldown = actors[index].weapon.cooldown();
        let (attacker, victim) = pair_mut(actors, index, target);
        if let Some(hit) = resolve_hit(attacker, victim, rng) {
            attacker.attack_cd = cooldown;
            events.push(format!(
                "{} hiere a {} ({})",
                attacker.name(),
                victim.name(),
                hit.limb.name()
            ));
            if victim.life == Life::Dead {
                events.push(format!("{} mata a {}", attacker.name(), victim.name()));
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::{Actor, AiState, Role};
    use crate::world::{Map, Rng};
    use macroquad::prelude::*;

    fn at(id: usize, role: Role, x: i32, y: i32) -> Actor {
        Actor::new(
            id,
            vec2(x as f32 * 32.0 + 16.0, y as f32 * 32.0 + 16.0),
            role,
        )
    }

    fn open_map() -> Map {
        Map::from_rows(&[
            "##########",
            "#........#",
            "#........#",
            "#........#",
            "#........#",
            "#........#",
            "#........#",
            "#........#",
            "##########",
        ])
    }

    fn run(actors: &mut Vec<Actor>, map: &Map, seconds: f32) {
        let mut rng = Rng::new(5);
        for _ in 0..(seconds * 60.0) as i32 {
            tick(actors, map, 1.0 / 60.0, &mut rng);
        }
    }

    #[test]
    fn guard_hates_bandit() {
        let guard = at(0, Role::Guard, 1, 1);
        let bandit = at(1, Role::Bandit, 2, 2);
        assert!(hostile(&guard, &bandit));
        assert!(hostile(&bandit, &guard));
    }

    #[test]
    fn town_is_neutral_to_player() {
        let guard = at(0, Role::Guard, 1, 1);
        let player = at(1, Role::Player, 2, 2);
        assert!(!hostile(&guard, &player));
    }

    #[test]
    fn town_turns_on_player_after_attack() {
        let mut guard = at(0, Role::Guard, 1, 1);
        let player = at(1, Role::Player, 2, 2);
        guard.hostile_to_player = true;
        assert!(hostile(&guard, &player));
    }

    #[test]
    fn civilian_flees_from_bandit() {
        let map = open_map();
        let mut actors = vec![at(0, Role::Civilian, 5, 5), at(1, Role::Bandit, 7, 5)];
        let before = (actors[1].pos - actors[0].pos).length();
        run(&mut actors, &map, 0.2);
        assert_eq!(actors[0].ai, AiState::Flee);
        let after = (actors[1].pos - actors[0].pos).length();
        assert!(after >= before - 1.0, "antes {before}, despues {after}");
    }

    #[test]
    fn dog_attacks_player_in_range() {
        let map = open_map();
        let mut actors = vec![at(0, Role::Player, 5, 5), at(1, Role::Dog, 7, 5)];
        run(&mut actors, &map, 0.2);
        assert_eq!(actors[1].ai, AiState::Chase);
    }

    #[test]
    fn unreachable_target_stays_idle() {
        let map = Map::from_rows(&["#######", "#.....#", "#.###.#", "#.#.#.#", "#######"]);
        let mut actors = vec![at(0, Role::Bandit, 1, 1), at(1, Role::Player, 3, 3)];
        run(&mut actors, &map, 0.9);
        assert_eq!(actors[0].ai, AiState::Idle);
    }

    #[test]
    fn bandit_approaches_target() {
        let map = open_map();
        let mut actors = vec![at(0, Role::Bandit, 2, 2), at(1, Role::Player, 7, 2)];
        let before = (actors[1].pos - actors[0].pos).length();
        run(&mut actors, &map, 2.0);
        let after = (actors[1].pos - actors[0].pos).length();
        assert!(after < before - 100.0, "antes {before}, despues {after}");
    }
}

