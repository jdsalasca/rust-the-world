use crate::actor::{Actor, Life, Limb};
use crate::world::{Map, Rng};
use macroquad::prelude::*;

pub struct Hit {
    pub limb: Limb,
    pub damage: f32,
}

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

/// Golpe cuerpo a cuerpo si el objetivo está en alcance y no hay roca en medio.
/// Un objetivo caído muere.
pub fn resolve_hit(attacker: &Actor, target: &mut Actor, rng: &mut Rng, map: &Map) -> Option<Hit> {
    if attacker.life != Life::Up || target.life == Life::Dead {
        return None;
    }
    if (target.pos - attacker.pos).length() > attacker.weapon.reach() {
        return None;
    }
    if map.line_blocked(attacker.pos, target.pos) {
        return None;
    }
    let damage = attacker.damage();
    let limb = target.hurt(damage, rng)?;
    Some(Hit { limb, damage })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::{Role, Wound};

    fn attacker() -> Actor {
        Actor::new(1, vec2(48., 48.), Role::Guard)
    }

    fn open_map() -> Map {
        Map::from_rows(&[
            "########", "#......#", "#......#", "#......#", "#......#", "########",
        ])
    }

    #[test]
    fn out_of_reach_misses() {
        let attacker = attacker();
        let mut target = Actor::new(2, vec2(128., 48.), Role::Bandit);
        assert!(resolve_hit(&attacker, &mut target, &mut Rng::new(1), &open_map()).is_none());
        assert!((target.hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn hit_damages_and_wounds() {
        let attacker = attacker();
        assert_eq!(attacker.weapon, crate::actor::Weapon::Sword);
        let mut target = Actor::new(2, vec2(68., 48.), Role::Bandit);
        let hit =
            resolve_hit(&attacker, &mut target, &mut Rng::new(1), &open_map()).expect("golpe");
        assert!((hit.damage - 11.0).abs() < 0.01);
        assert!((target.hp - 89.0).abs() < 0.01);
    }

    #[test]
    fn hit_on_downed_executes() {
        let attacker = attacker();
        let mut target = Actor::new(2, vec2(68., 48.), Role::Bandit);
        target.life = Life::Downed;
        target.hp = 0.0;
        assert!(resolve_hit(&attacker, &mut target, &mut Rng::new(1), &open_map()).is_some());
        assert_eq!(target.life, Life::Dead);
    }

    #[test]
    fn cannot_hit_through_rock() {
        let map = Map::from_rows(&["######", "#..###", "#....#", "######"]);
        let attacker = Actor::new(1, vec2(80., 48.), Role::Guard);
        let mut target = Actor::new(2, vec2(112., 72.), Role::Bandit);
        assert!(resolve_hit(&attacker, &mut target, &mut Rng::new(1), &map).is_none());
        assert!((target.hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn arc_ignores_target_behind() {
        let attacker = attacker();
        let behind = Actor::new(2, vec2(28., 48.), Role::Bandit);
        assert!(!in_arc(&attacker, &behind, vec2(1., 0.)));
    }

    #[test]
    fn arc_accepts_target_in_front() {
        let attacker = attacker();
        let front = Actor::new(2, vec2(68., 58.), Role::Bandit);
        assert!(in_arc(&attacker, &front, vec2(1., 0.)));
    }

    #[test]
    fn severe_wound_bleeds() {
        let mut target = Actor::new(2, vec2(68., 48.), Role::Bandit);
        target.wounds[0] = Wound::Severe;
        assert!(target.is_bleeding());
    }
}
