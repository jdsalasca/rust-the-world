use crate::actor::Role;
use crate::world::Tile;
use macroquad::prelude::*;

pub const FRAME: f32 = 32.0;

macro_rules! texture {
    ($path:literal) => {{
        let mut texture = Texture2D::from_file_with_format(
            include_bytes!(concat!("../assets/art/", $path)),
            Some(ImageFormat::Png),
        );
        texture.set_filter(FilterMode::Nearest);
        texture
    }};
}

pub struct CharSheet {
    pub idle: Texture2D,
    pub walk: Texture2D,
    pub attack: Texture2D,
}

#[derive(Clone, Copy)]
pub enum AnimKind {
    Idle,
    Walk,
    Attack,
}

#[derive(Clone, Copy)]
pub enum Prop {
    Cactus,
    Rock,
    Tent,
    Fire,
    Crate,
    Well,
}

pub struct Art {
    pub player: CharSheet,
    pub bandit: CharSheet,
    pub guard: CharSheet,
    pub merchant: CharSheet,
    pub civilian: CharSheet,
    pub dog: CharSheet,
    pub sand: Texture2D,
    pub dirt: Texture2D,
    pub road: Texture2D,
    pub rock_tile: Texture2D,
    pub water: Texture2D,
    pub cactus: Texture2D,
    pub rock_prop: Texture2D,
    pub tent: Texture2D,
    pub fire: Texture2D,
    pub box_prop: Texture2D,
    pub well: Texture2D,
    pub food: Texture2D,
    pub bandage: Texture2D,
    pub coin: Texture2D,
    pub sword: Texture2D,
}

impl Art {
    pub fn load() -> Art {
        Art {
            player: CharSheet {
                idle: texture!("chars/player_idle.png"),
                walk: texture!("chars/player_walk.png"),
                attack: texture!("chars/player_attack.png"),
            },
            bandit: CharSheet {
                idle: texture!("chars/bandit_idle.png"),
                walk: texture!("chars/bandit_walk.png"),
                attack: texture!("chars/bandit_attack.png"),
            },
            guard: CharSheet {
                idle: texture!("chars/guard_idle.png"),
                walk: texture!("chars/guard_walk.png"),
                attack: texture!("chars/guard_attack.png"),
            },
            merchant: CharSheet {
                idle: texture!("chars/merchant_idle.png"),
                walk: texture!("chars/merchant_walk.png"),
                attack: texture!("chars/merchant_attack.png"),
            },
            civilian: CharSheet {
                idle: texture!("chars/civilian_idle.png"),
                walk: texture!("chars/civilian_walk.png"),
                attack: texture!("chars/civilian_attack.png"),
            },
            dog: CharSheet {
                idle: texture!("chars/dog_idle.png"),
                walk: texture!("chars/dog_walk.png"),
                attack: texture!("chars/dog_attack.png"),
            },
            sand: texture!("tiles/sand.png"),
            dirt: texture!("tiles/dirt.png"),
            road: texture!("tiles/road.png"),
            rock_tile: texture!("tiles/rock.png"),
            water: texture!("tiles/water.png"),
            cactus: texture!("props/cactus.png"),
            rock_prop: texture!("props/rock.png"),
            tent: texture!("props/tent.png"),
            fire: texture!("props/fire.png"),
            box_prop: texture!("props/crate.png"),
            well: texture!("props/well.png"),
            food: texture!("items/food.png"),
            bandage: texture!("items/bandage.png"),
            coin: texture!("items/coin.png"),
            sword: texture!("items/sword.png"),
        }
    }

    pub fn sheet(&self, role: Role) -> &CharSheet {
        match role {
            Role::Player => &self.player,
            Role::Bandit => &self.bandit,
            Role::Guard => &self.guard,
            Role::Merchant => &self.merchant,
            Role::Civilian => &self.civilian,
            Role::Dog => &self.dog,
        }
    }
}

fn tile_texture(art: &Art, tile: Tile) -> &Texture2D {
    match tile {
        Tile::Sand => &art.sand,
        Tile::Dirt => &art.dirt,
        Tile::Road => &art.road,
        Tile::Rock => &art.rock_tile,
        Tile::Water => &art.water,
    }
}

pub fn draw_tile(art: &Art, tile: Tile, at: Vec2) {
    draw_texture(tile_texture(art, tile), at.x, at.y, WHITE);
}

pub fn draw_prop(art: &Art, prop: Prop, at: Vec2) {
    let texture = match prop {
        Prop::Cactus => &art.cactus,
        Prop::Rock => &art.rock_prop,
        Prop::Tent => &art.tent,
        Prop::Fire => &art.fire,
        Prop::Crate => &art.box_prop,
        Prop::Well => &art.well,
    };
    draw_texture(texture, at.x, at.y, WHITE);
}

pub fn draw_item(art: &Art, item: crate::economy::Item, at: Vec2) {
    let texture = match item {
        crate::economy::Item::Food => &art.food,
        crate::economy::Item::Bandage => &art.bandage,
        crate::economy::Item::Coin(_) => &art.coin,
        crate::economy::Item::Weapon(_) => &art.sword,
    };
    draw_texture(texture, at.x - 8.0, at.y - 8.0, WHITE);
}

pub fn draw_actor(sheet: &CharSheet, anim: AnimKind, time: f32, pos: Vec2, facing_right: bool) {
    let texture = match anim {
        AnimKind::Idle => &sheet.idle,
        AnimKind::Walk => &sheet.walk,
        AnimKind::Attack => &sheet.attack,
    };
    let frames = (texture.width() / FRAME).max(1.0);
    let fps = match anim {
        AnimKind::Idle => 2.0,
        AnimKind::Walk => 8.0,
        AnimKind::Attack => 9.0,
    };
    let frame = (time * fps) as i32 % frames as i32;
    let source = Rect::new(frame as f32 * FRAME, 0.0, FRAME, FRAME);
    draw_texture_ex(
        texture,
        pos.x - FRAME / 2.0,
        pos.y - FRAME + 4.0,
        WHITE,
        DrawTextureParams {
            source: Some(source),
            flip_x: !facing_right,
            ..Default::default()
        },
    );
}

