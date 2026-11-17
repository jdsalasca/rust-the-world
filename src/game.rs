use crate::actor::{Actor, BODY_RADIUS, Faction, Life, Role, Weapon};
use crate::ai;
use crate::combat;
use crate::economy::{self, Drop};
use crate::log::EventLog;
use crate::world::{Map, Rng, TILE_SIZE};
use macroquad::prelude::*;

pub const DEFAULT_SEED: u64 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Shop,
    Dead,
}

pub struct Game {
    pub map: Map,
    pub actors: Vec<Actor>,
    pub drops: Vec<Drop>,
    pub log: EventLog,
    pub state: GameState,
    pub paused: bool,
    pub speed: f32,
    pub rng: Rng,
    pub player: usize,
    pub seed: u64,
}

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

impl Game {
    pub fn new(seed: u64) -> Game {
        let map = Map::generate(seed);
        let mut actors = spawn_actors(&map);
        if let Some(index) = actors.iter().rposition(|body| body.role == Role::Bandit) {
            actors[index].weapon = Weapon::Axe;
        }
        Game {
            map,
            actors,
            drops: Vec::new(),
            log: EventLog::new(),
            state: GameState::Playing,
            paused: false,
            speed: 1.0,
            rng: Rng::new(seed + 11),
            player: 0,
            seed,
        }
    }

    pub fn restart(&mut self) {
        *self = Game::new(self.seed);
    }

    pub fn player(&self) -> &Actor {
        &self.actors[self.player]
    }

    pub fn tick(&mut self, dt: f32) {
        if self.state != GameState::Playing || self.paused {
            return;
        }
        let dt = dt * self.speed;

        for index in 0..self.actors.len() {
            let events = {
                let body = &mut self.actors[index];
                body.moving = false;
                body.anim_time += dt;
                body.attack_cd = (body.attack_cd - dt).max(0.0);
                body.tick(dt)
            };
            for event in events {
                self.log.push(event);
            }
        }

        let events = ai::tick(&mut self.actors, &self.map, dt, &mut self.rng);
        for event in events {
            self.log.push(event);
        }

        let player = self.player;
        let events = economy::pick_up(&mut self.actors[player], &mut self.drops);
        for event in events {
            self.log.push(event);
        }

        for index in 0..self.actors.len() {
            if self.actors[index].life == Life::Dead && !self.actors[index].death_handled {
                let name = self.actors[index].name();
                if self.actors[index].role == Role::Player {
                    self.log.push("Has muerto".to_owned());
                } else {
                    self.log.push(format!("Ha muerto el {name}"));
                }
                self.drops
                    .extend(economy::drop_loot(&mut self.actors[index]));
            }
        }

        if self.actors[self.player].life == Life::Dead {
            self.state = GameState::Dead;
        }
    }

    pub fn attack(&mut self, aim: Vec2) {
        if self.state != GameState::Playing || self.actors[self.player].life != Life::Up {
            return;
        }
        if self.actors[self.player].attack_cd > 0.0 {
            return;
        }

        let player = self.player;
        let mut best: Option<(usize, f32)> = None;
        for (index, other) in self.actors.iter().enumerate().skip(1) {
            if other.life == Life::Dead {
                continue;
            }
            let offset = other.pos - self.actors[player].pos;
            let distance = offset.length();
            if distance > self.actors[player].weapon.reach() {
                continue;
            }
            if aim != Vec2::ZERO && !combat::in_arc(&self.actors[player], other, aim) {
                continue;
            }
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((index, distance));
            }
        }

        let Some((index, _)) = best else {
            return;
        };
        let cooldown = self.actors[player].weapon.cooldown();
        let hit = {
            let (before, after) = self.actors.split_at_mut(index);
            combat::resolve_hit(&before[player], &mut after[0], &mut self.rng)
        };
        self.actors[player].attack_cd = cooldown;
        if let Some(hit) = hit {
            self.log.push(format!(
                "Golpeas al {} ({}) por {:.0}",
                self.actors[index].name(),
                hit.limb.name(),
                hit.damage
            ));
            self.actors[index].hostile_to_player = true;
            self.actors[index].target = Some(player);
            self.actors[index].think = 0.5;
            if self.actors[index].faction == Faction::Town {
                for body in self.actors.iter_mut() {
                    if body.faction == Faction::Town {
                        body.hostile_to_player = true;
                    }
                }
            }
        }
    }

    pub fn eat(&mut self) -> bool {
        let player = self.player;
        if economy::eat(&mut self.actors[player]) {
            self.log.push("Comes algo".to_owned());
            true
        } else {
            false
        }
    }

    pub fn bandage(&mut self) -> bool {
        let player = self.player;
        if self.actors[player].bandage() {
            self.log.push("Vendas tus heridas".to_owned());
            true
        } else {
            false
        }
    }

    pub fn merchant_near(&self) -> Option<usize> {
        let player = self.player;
        self.actors
            .iter()
            .position(|other| other.id != player && economy::can_trade(&self.actors[player], other))
    }

    pub fn open_shop(&mut self) -> bool {
        if self.state != GameState::Playing || self.merchant_near().is_none() {
            return false;
        }
        self.state = GameState::Shop;
        true
    }

    pub fn close_shop(&mut self) {
        if self.state == GameState::Shop {
            self.state = GameState::Playing;
        }
    }

    pub fn buy_food(&mut self) -> bool {
        let player = self.player;
        if economy::buy_food(&mut self.actors[player]) {
            self.log.push("Compras comida".to_owned());
            true
        } else {
            false
        }
    }

    pub fn buy_bandage(&mut self) -> bool {
        let player = self.player;
        if economy::buy_bandage(&mut self.actors[player]) {
            self.log.push("Compras una venda".to_owned());
            true
        } else {
            false
        }
    }

    pub fn sell_spare(&mut self) -> bool {
        let player = self.player;
        if economy::sell_spare(&mut self.actors[player]) {
            self.log.push("Vendes el arma de repuesto".to_owned());
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_resets_world() {
        let mut game = Game::new(7);
        let tiles = game.map.tiles.clone();
        game.actors[0].hp = 12.0;
        game.actors[1].life = Life::Dead;
        game.restart();
        assert_eq!(game.actors.len(), 13);
        assert_eq!(game.map.tiles, tiles);
        assert!((game.actors[0].hp - 100.0).abs() < 0.01);
        assert_eq!(game.actors[1].life, Life::Up);
    }

    #[test]
    fn player_death_sets_state() {
        let mut game = Game::new(7);
        game.actors[0].life = Life::Dead;
        game.tick(0.1);
        assert_eq!(game.state, GameState::Dead);
    }

    #[test]
    fn paused_world_does_not_tick() {
        let mut game = Game::new(7);
        game.paused = true;
        let hunger = game.actors[0].hunger;
        game.tick(10.0);
        assert!((game.actors[0].hunger - hunger).abs() < 0.001);
    }

    #[test]
    fn bandit_and_guard_fight_on_their_own() {
        let mut game = Game::new(7);
        let guard = game
            .actors
            .iter()
            .position(|body| body.role == Role::Guard)
            .unwrap();
        let bandit = game
            .actors
            .iter()
            .position(|body| body.role == Role::Bandit)
            .unwrap();
        game.actors[bandit].pos = game.actors[guard].pos + vec2(80., 0.);
        for _ in 0..600 {
            game.tick(1.0 / 60.0);
        }
        assert!(
            game.actors[guard].hp < 100.0 || game.actors[bandit].hp < 100.0,
            "nadie se peleo: guardia {} bandido {}",
            game.actors[guard].hp,
            game.actors[bandit].hp
        );
    }

    #[test]
    fn player_attack_hurts_nearest_bandit() {
        let mut game = Game::new(7);
        let bandit = game
            .actors
            .iter()
            .position(|body| body.role == Role::Bandit)
            .unwrap();
        game.actors[bandit].pos = game.actors[0].pos + vec2(20., 0.);
        game.attack(vec2(1., 0.));
        assert!(game.actors[bandit].hp < 100.0);
        assert!(game.actors[bandit].hostile_to_player);
    }
}
