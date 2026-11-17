use crate::world::Rng;
use macroquad::prelude::*;

pub const MAX_HP: f32 = 100.0;
pub const MAX_HUNGER: f32 = 100.0;
pub const HUNGER_PER_SEC: f32 = 1.0 / 15.0;
pub const HEAL_SECS: f32 = 90.0;
pub const WAKE_SECS: f32 = 15.0;
pub const BLEED_PER_SEC: f32 = 0.5;
pub const EXECUTE_BELOW: f32 = -20.0;
pub const MOVE_SPEED: f32 = 120.0;
pub const BODY_RADIUS: f32 = 9.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Faction {
    Player,
    Town,
    Bandit,
    Wildlife,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Player,
    Guard,
    Merchant,
    Civilian,
    Bandit,
    Dog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limb {
    ArmL,
    ArmR,
    LegL,
    LegR,
}

impl Limb {
    pub const ALL: [Limb; 4] = [Limb::ArmL, Limb::ArmR, Limb::LegL, Limb::LegR];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Limb::ArmL => "brazo izquierdo",
            Limb::ArmR => "brazo derecho",
            Limb::LegL => "pierna izquierda",
            Limb::LegR => "pierna derecha",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wound {
    None,
    Light,
    Severe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Life {
    Up,
    Downed,
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Flee,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    Fists,
    Dagger,
    Sword,
    Axe,
}

impl Weapon {
    pub fn damage(self) -> f32 {
        match self {
            Weapon::Fists => 6.0,
            Weapon::Dagger => 8.0,
            Weapon::Sword => 11.0,
            Weapon::Axe => 14.0,
        }
    }

    pub fn reach(self) -> f32 {
        match self {
            Weapon::Fists => 34.0,
            Weapon::Dagger => 40.0,
            Weapon::Sword => 44.0,
            Weapon::Axe => 46.0,
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Weapon::Fists => 0.5,
            Weapon::Dagger => 0.6,
            Weapon::Sword => 0.8,
            Weapon::Axe => 1.0,
        }
    }
}

pub struct Actor {
    pub id: usize,
    pub pos: Vec2,
    pub faction: Faction,
    pub role: Role,
    pub hp: f32,
    pub hunger: f32,
    pub wounds: [Wound; 4],
    pub life: Life,
    pub downed: f32,
    pub ai: AiState,
    pub target: Option<usize>,
    pub hostile_to_player: bool,
    pub path: Vec<IVec2>,
    pub attack_cd: f32,
    pub facing_right: bool,
    pub home: Vec2,
    pub repath: f32,
    pub think: f32,
    pub flee_timer: f32,
    pub give_up: f32,
    pub food: u32,
    pub bandages: u32,
    pub coins: u32,
    pub weapon: Weapon,
    pub spare: Option<Weapon>,
    pub heal: [f32; 4],
    pub death_handled: bool,
    pub anim_time: f32,
    pub moving: bool,
}

impl Actor {
    pub fn new(id: usize, pos: Vec2, role: Role) -> Actor {
        let faction = match role {
            Role::Player => Faction::Player,
            Role::Guard | Role::Merchant | Role::Civilian => Faction::Town,
            Role::Bandit => Faction::Bandit,
            Role::Dog => Faction::Wildlife,
        };
        let weapon = match role {
            Role::Guard => Weapon::Sword,
            Role::Bandit => Weapon::Dagger,
            _ => Weapon::Fists,
        };
        Actor {
            id,
            pos,
            faction,
            role,
            hp: MAX_HP,
            hunger: MAX_HUNGER,
            wounds: [Wound::None; 4],
            life: Life::Up,
            downed: 0.0,
            ai: AiState::Idle,
            target: None,
            hostile_to_player: false,
            path: Vec::new(),
            attack_cd: 0.0,
            facing_right: true,
            home: pos,
            repath: 0.0,
            think: 0.0,
            flee_timer: 0.0,
            give_up: 0.0,
            food: 0,
            bandages: 0,
            coins: 0,
            weapon,
            spare: None,
            heal: [0.0; 4],
            death_handled: false,
            anim_time: 0.0,
            moving: false,
        }
    }

    fn wounds_of(&self, arms: bool) -> f32 {
        let limbs = if arms {
            [Limb::ArmL, Limb::ArmR]
        } else {
            [Limb::LegL, Limb::LegR]
        };
        limbs
            .iter()
            .filter(|limb| self.wounds[limb.index()] != Wound::None)
            .count() as f32
    }

    pub fn speed(&self) -> f32 {
        if self.life != Life::Up {
            return 0.0;
        }
        MOVE_SPEED * (1.0 - 0.15 * self.wounds_of(false))
    }

    pub fn damage(&self) -> f32 {
        self.weapon.damage() * (1.0 - 0.15 * self.wounds_of(true))
    }

    pub fn is_bleeding(&self) -> bool {
        self.wounds.iter().any(|wound| *wound == Wound::Severe)
    }

    pub fn bandage(&mut self) -> bool {
        if self.bandages == 0 || !self.is_bleeding() {
            return false;
        }
        self.bandages -= 1;
        for index in 0..4 {
            if self.wounds[index] == Wound::Severe {
                self.wounds[index] = Wound::Light;
                self.heal[index] = 0.0;
            }
        }
        true
    }

    /// Aplica daño a una zona al azar. `None` si el objetivo ya está muerto.
    pub fn hurt(&mut self, amount: f32, rng: &mut Rng) -> Option<Limb> {
        if self.life == Life::Dead || amount <= 0.0 {
            return None;
        }
        let limb = Limb::ALL[rng.range(0, 3) as usize];
        if self.life == Life::Downed {
            self.life = Life::Dead;
            return Some(limb);
        }
        self.hp -= amount;
        if amount > 20.0 || rng.chance(0.12) {
            self.wounds[limb.index()] = Wound::Severe;
        } else if self.wounds[limb.index()] == Wound::None {
            self.wounds[limb.index()] = Wound::Light;
        }
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.life = Life::Downed;
            self.downed = 0.0;
        }
        Some(limb)
    }

    pub fn tick(&mut self, dt: f32) -> Vec<String> {
        let mut events = Vec::new();
        if self.life == Life::Dead {
            return events;
        }

        match self.life {
            Life::Up => {
                if self.hunger > 0.0 {
                    self.hunger = (self.hunger - HUNGER_PER_SEC * dt).max(0.0);
                } else {
                    self.hp -= dt;
                }
                if self.is_bleeding() {
                    self.hp -= BLEED_PER_SEC * dt;
                }
                for index in 0..4 {
                    if self.wounds[index] == Wound::Light {
                        self.heal[index] += dt;
                        if self.heal[index] >= HEAL_SECS {
                            self.wounds[index] = Wound::None;
                            self.heal[index] = 0.0;
                            events.push("Una herida ha curado".to_owned());
                        }
                    }
                }
                if self.hp <= 0.0 {
                    self.hp = 0.0;
                    self.life = Life::Downed;
                    self.downed = 0.0;
                    events.push(format!("{} cae al suelo", self.name()));
                }
            }
            Life::Downed => {
                if self.is_bleeding() {
                    self.hp -= BLEED_PER_SEC * dt;
                } else {
                    self.downed += dt;
                    if self.downed >= WAKE_SECS {
                        if self.role == Role::Player {
                            self.life = Life::Dead;
                            events.push("Te desangras en el suelo".to_owned());
                        } else {
                            self.life = Life::Up;
                            self.hp = 10.0;
                            self.downed = 0.0;
                            events.push(format!("{} se levanta", self.name()));
                        }
                    }
                }
                if self.hp <= EXECUTE_BELOW {
                    self.life = Life::Dead;
                }
            }
            Life::Dead => {}
        }
        events
    }

    pub fn name(&self) -> &'static str {
        match self.role {
            Role::Player => "tú",
            Role::Guard => "el guardia",
            Role::Merchant => "el mercader",
            Role::Civilian => "el civil",
            Role::Bandit => "el bandido",
            Role::Dog => "el perro",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn npc(role: Role) -> Actor {
        Actor::new(0, vec2(100., 100.), role)
    }

    #[test]
    fn hunger_drops_one_per_fifteen_seconds() {
        let mut actor = npc(Role::Player);
        actor.tick(15.0);
        assert!((actor.hunger - 99.0).abs() < 0.01, "hunger = {}", actor.hunger);
    }

    #[test]
    fn starvation_damages() {
        let mut actor = npc(Role::Player);
        actor.hunger = 0.0;
        actor.tick(10.0);
        assert!((actor.hp - 90.0).abs() < 0.01, "hp = {}", actor.hp);
    }

    #[test]
    fn leg_wound_slows() {
        let mut actor = npc(Role::Player);
        let base = actor.speed();
        actor.wounds[Limb::LegL.index()] = Wound::Light;
        assert!((actor.speed() - base * 0.85).abs() < 0.01);
    }

    #[test]
    fn arm_wound_weakens() {
        let mut actor = npc(Role::Player);
        let base = actor.damage();
        actor.wounds[Limb::ArmR.index()] = Wound::Light;
        assert!((actor.damage() - base * 0.85).abs() < 0.01);
    }

    #[test]
    fn bandage_stops_bleeding() {
        let mut actor = npc(Role::Player);
        actor.wounds[Limb::ArmL.index()] = Wound::Severe;
        actor.bandages = 1;
        assert!(actor.is_bleeding());
        assert!(actor.bandage());
        assert!(!actor.is_bleeding());
        assert_eq!(actor.bandages, 0);
        assert!(!actor.bandage());
    }

    #[test]
    fn wound_heals_after_ninety_seconds() {
        let mut actor = npc(Role::Player);
        actor.wounds[Limb::ArmL.index()] = Wound::Severe;
        actor.bandages = 1;
        actor.bandage();
        actor.tick(90.0);
        assert_eq!(actor.wounds[Limb::ArmL.index()], Wound::None);
    }

    #[test]
    fn zero_hp_goes_downed_not_dead() {
        let mut actor = npc(Role::Player);
        let mut rng = Rng::new(3);
        actor.hurt(200.0, &mut rng);
        assert_eq!(actor.life, Life::Downed);
    }

    #[test]
    fn downed_npc_wakes() {
        let mut actor = npc(Role::Guard);
        actor.life = Life::Downed;
        actor.hp = 0.0;
        actor.tick(15.0);
        assert_eq!(actor.life, Life::Up);
        assert!((actor.hp - 10.0).abs() < 0.01);
    }

    #[test]
    fn downed_player_does_not_wake() {
        let mut actor = npc(Role::Player);
        actor.life = Life::Downed;
        actor.hp = 0.0;
        actor.tick(16.0);
        assert_eq!(actor.life, Life::Dead);
    }

    #[test]
    fn downed_bleeding_dies() {
        let mut actor = npc(Role::Guard);
        actor.life = Life::Downed;
        actor.hp = 0.0;
        actor.wounds[Limb::LegR.index()] = Wound::Severe;
        actor.tick(41.0);
        assert_eq!(actor.life, Life::Dead);
    }
}
