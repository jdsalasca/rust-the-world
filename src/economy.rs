use crate::actor::{Actor, Life, Weapon};
use macroquad::prelude::*;

pub const FOOD_PRICE: u32 = 5;
pub const BANDAGE_PRICE: u32 = 8;
pub const PICKUP_RADIUS: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
    Food,
    Bandage,
    Coin(u32),
    Weapon(Weapon),
}

#[derive(Clone, Copy, Debug)]
pub struct Drop {
    pub pos: Vec2,
    pub item: Item,
}

/// Botín de un cadáver. Devuelve vacío la segunda vez: `death_handled` lo impide.
pub fn drop_loot(actor: &mut Actor) -> Vec<Drop> {
    if actor.life != Life::Dead || actor.death_handled {
        return Vec::new();
    }
    actor.death_handled = true;
    let mut drops = vec![Drop {
        pos: actor.pos,
        item: Item::Coin(2 + (actor.id % 5) as u32),
    }];
    if actor.food > 0 {
        drops.push(Drop { pos: actor.pos, item: Item::Food });
    }
    if actor.bandages > 0 {
        drops.push(Drop { pos: actor.pos, item: Item::Bandage });
    }
    if actor.weapon != Weapon::Fists {
        drops.push(Drop { pos: actor.pos, item: Item::Weapon(actor.weapon) });
    }
    drops
}

pub fn pick_up(actor: &mut Actor, drops: &mut Vec<Drop>) -> Vec<String> {
    let mut events = Vec::new();
    drops.retain(|drop| {
        if (drop.pos - actor.pos).length() > PICKUP_RADIUS {
            return true;
        }
        match drop.item {
            Item::Food => {
                actor.food += 1;
                events.push("Recoges comida".to_owned());
            }
            Item::Bandage => {
                actor.bandages += 1;
                events.push("Recoges una venda".to_owned());
            }
            Item::Coin(amount) => {
                actor.coins += amount;
                events.push(format!("Recoges {amount} monedas"));
            }
            Item::Weapon(weapon) => {
                if weapon.damage() > actor.weapon.damage() {
                    actor.spare = Some(actor.weapon);
                    actor.weapon = weapon;
                    events.push("Equipas un arma mejor".to_owned());
                } else {
                    actor.spare = Some(weapon);
                    events.push("Guardas un arma de repuesto".to_owned());
                }
            }
        }
        false
    });
    events
}

pub fn buy_food(actor: &mut Actor) -> bool {
    if actor.coins < FOOD_PRICE {
        return false;
    }
    actor.coins -= FOOD_PRICE;
    actor.food += 1;
    true
}

pub fn buy_bandage(actor: &mut Actor) -> bool {
    if actor.coins < BANDAGE_PRICE {
        return false;
    }
    actor.coins -= BANDAGE_PRICE;
    actor.bandages += 1;
    true
}

pub fn sell_spare(actor: &mut Actor) -> bool {
    match actor.spare.take() {
        Some(weapon) => {
            actor.coins += (weapon.damage() / 2.0) as u32;
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::{Role, Weapon};
    use macroquad::prelude::*;

    #[test]
    fn loot_drops_once() {
        let mut bandit = Actor::new(3, vec2(10., 10.), Role::Bandit);
        bandit.food = 1;
        bandit.bandages = 1;
        bandit.life = Life::Dead;
        assert!(!drop_loot(&mut bandit).is_empty());
        assert!(drop_loot(&mut bandit).is_empty());
    }

    #[test]
    fn pickup_adds_items() {
        let mut player = Actor::new(0, vec2(50., 50.), Role::Player);
        let mut drops = vec![
            Drop { pos: vec2(52., 50.), item: Item::Food },
            Drop { pos: vec2(50., 54.), item: Item::Coin(4) },
            Drop { pos: vec2(200., 200.), item: Item::Bandage },
        ];
        let events = pick_up(&mut player, &mut drops);
        assert_eq!(player.food, 1);
        assert_eq!(player.coins, 4);
        assert_eq!(player.bandages, 0);
        assert_eq!(drops.len(), 1);
        assert!(!events.is_empty());
    }

    #[test]
    fn buy_food_needs_coins() {
        let mut player = Actor::new(0, Vec2::ZERO, Role::Player);
        player.coins = 5;
        assert!(buy_food(&mut player));
        assert_eq!(player.food, 1);
        assert_eq!(player.coins, 0);
        assert!(!buy_food(&mut player));
        player.coins = 4;
        assert!(!buy_food(&mut player));
    }

    #[test]
    fn sell_spare_pays_half() {
        let mut player = Actor::new(0, Vec2::ZERO, Role::Player);
        player.spare = Some(Weapon::Sword);
        assert!(sell_spare(&mut player));
        assert_eq!(player.coins, 5);
        assert!(!sell_spare(&mut player));
    }
}
