use crate::actor::{Actor, Life, Limb, Wound};
use crate::world::Rng;
use macroquad::prelude::*;

pub struct Hit {
    pub limb: Limb,
    pub damage: f32,
}

pub const SEVERE_DAMAGE: f32 = 20.0;

/// Cono frontal de ~120° alrededor de la dirección de ataque.
pub fn in_arc(attacker: &Actor, target: &Actor, aim: Vec2) -> bool {
    if aim == Vec2::ZERO {
        return true;
    }
    let offset = target.pos - attacker.pos;
    if offset == Vec2::ZERO {
        return true;
    }
    aim.normalize().dot(offset.normalize()) >= 0.5
}

/// Golpe cuerpo a cuerpo si el objetivo está en alcance. Un objetivo caído muere.
pub fn resolve_hit(attacker: &Actor, target: &mut Actor, rng: &mut Rng) -> Option<Hit> {
    if attacker.life != Life::Up || target.life == Life::Dead {
        return None;
    }
    if (target.pos - attacker.pos).length() > attacker.weapon.reach() {
        return None;
    }
    let damage = attacker.damage();
    let limb = target.hurt(damage, rng)?;
    if damage > SEVERE_DAMAGE && target.wounds[limb.index()] != Wound::Severe {
        target.wounds[limb.index()] = Wound::Severe;
    }
    Some(Hit { limb, damage })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::{Role, Weapon};
    use macroquad::prelude::*;

    fn attacker() -> Actor {
        Actor::new(1, Vec2::ZERO, Role::Guard)
    }

    #[test]
    fn out_of_reach_misses() {
        let attacker = attacker();
        let mut target = Actor::new(2, vec2(80., 0.), Role::Bandit);
        assert!(resolve_hit(&attacker, &mut target, &mut Rng::new(1)).is_none());
        assert!((target.hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn hit_damages_and_wounds() {
        let attacker = attacker();
        assert_eq!(attacker.weapon, Weapon::Sword);
        let mut target = Actor::new(2, vec2(20., 0.), Role::Bandit);
        let hit = resolve_hit(&attacker, &mut target, &mut Rng::new(1)).expect("golpe");
        assert!((hit.damage - 11.0).abs() < 0.01);
        assert!((target.hp - 89.0).abs() < 0.01);
    }

    #[test]
    fn hit_on_downed_executes() {
        let attacker = attacker();
        let mut target = Actor::new(2, vec2(20., 0.), Role::Bandit);
        target.life = Life::Downed;
        target.hp = 0.0;
        assert!(resolve_hit(&attacker, &mut target, &mut Rng::new(1)).is_some());
        assert_eq!(target.life, Life::Dead);
    }

    #[test]
    fn arc_ignores_target_behind() {
        let attacker = attacker();
        let behind = Actor::new(2, vec2(-20., 0.), Role::Bandit);
        assert!(!in_arc(&attacker, &behind, vec2(1., 0.)));
    }

    #[test]
    fn arc_accepts_target_in_front() {
        let attacker = attacker();
        let front = Actor::new(2, vec2(20., 10.), Role::Bandit);
        assert!(in_arc(&attacker, &front, vec2(1., 0.)));
    }
}
