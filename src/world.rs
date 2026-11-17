use macroquad::prelude::*;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_W: i32 = 64;
pub const MAP_H: i32 = 48;
pub const VIEW_W: f32 = 20.0 * TILE_SIZE;
pub const VIEW_H: f32 = 11.25 * TILE_SIZE;
pub const TOWN: (i32, i32) = (32, 12);
pub const CAMP: (i32, i32) = (10, 38);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Sand,
    Dirt,
    Road,
    Rock,
    Water,
}

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo + 1) as u32;
        lo + (self.next_u32() % span) as i32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.next_u32() as f32 / u32::MAX as f32 <= p
    }
}

pub struct Map {
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn generate(seed: u64) -> Map {
        let mut rng = Rng::new(seed);
        let mut map = Map { tiles: vec![Tile::Sand; (MAP_W * MAP_H) as usize] };

        for x in 0..MAP_W {
            for y in 0..MAP_H {
                if x == 0 || y == 0 || x == MAP_W - 1 || y == MAP_H - 1 {
                    map.set(x, y, Tile::Rock);
                }
            }
        }

        map.rect(TOWN.0 - 5, TOWN.1 - 4, 11, 9, Tile::Dirt);
        map.rect(CAMP.0 - 4, CAMP.1 - 3, 9, 7, Tile::Dirt);

        map.rect(TOWN.0 - 1, TOWN.1, 3, CAMP.1 - TOWN.1 + 2, Tile::Road);
        map.rect(CAMP.0, CAMP.1 - 1, TOWN.0 - CAMP.0 + 1, 3, Tile::Road);

        let (px, py) = (52, 30);
        for x in px - 4..=px + 4 {
            for y in py - 3..=py + 3 {
                let dx = (x - px) as f32 / 4.0;
                let dy = (y - py) as f32 / 3.0;
                if dx * dx + dy * dy <= 1.0 {
                    map.set(x, y, Tile::Water);
                }
            }
        }

        let mut rocks = 0;
        while rocks < 40 {
            let x = rng.range(2, MAP_W - 3);
            let y = rng.range(2, MAP_H - 3);
            if map.tile(x, y) == Tile::Sand {
                map.set(x, y, Tile::Rock);
                rocks += 1;
            }
        }

        map
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && x < MAP_W && y < MAP_H).then(|| (y * MAP_W + x) as usize)
    }

    fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if let Some(i) = self.index(x, y) {
            self.tiles[i] = tile;
        }
    }

    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, tile: Tile) {
        for ix in x..x + w {
            for iy in y..y + h {
                self.set(ix, iy, tile);
            }
        }
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        self.index(x, y).map_or(Tile::Rock, |i| self.tiles[i])
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        matches!(self.tile(x, y), Tile::Rock | Tile::Water)
    }

    pub fn blocked_at(&self, pos: Vec2) -> bool {
        self.blocked((pos.x / TILE_SIZE).floor() as i32, (pos.y / TILE_SIZE).floor() as i32)
    }

    pub fn town_spawn(&self) -> Vec2 {
        vec2(TOWN.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0, TOWN.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0)
    }

    pub fn camp_spawn(&self) -> Vec2 {
        vec2(CAMP.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0, CAMP.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0)
    }

    pub fn clamp_camera(&self, target: Vec2) -> Vec2 {
        vec2(
            target.x.clamp(VIEW_W / 2.0, MAP_W as f32 * TILE_SIZE - VIEW_W / 2.0),
            target.y.clamp(VIEW_H / 2.0, MAP_H as f32 * TILE_SIZE - VIEW_H / 2.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(Map::generate(7).tiles, Map::generate(7).tiles);
    }

    #[test]
    fn generation_has_all_terrain() {
        let map = Map::generate(7);
        assert!(map.tiles.contains(&Tile::Road));
        assert!(map.tiles.contains(&Tile::Water));
        assert!(map.tiles.contains(&Tile::Rock));
        assert!(map.tiles.contains(&Tile::Dirt));
    }

    #[test]
    fn border_is_blocked() {
        let map = Map::generate(7);
        assert!(map.blocked(0, 0));
        assert!(map.blocked(-3, 5));
        assert!(map.blocked(MAP_W, MAP_H));
    }

    #[test]
    fn camera_clamps_to_map() {
        let map = Map::generate(7);
        assert_eq!(map.clamp_camera(vec2(-500., -500.)), vec2(VIEW_W / 2., VIEW_H / 2.));
        assert_eq!(
            map.clamp_camera(vec2(99_999., 99_999.)),
            vec2(
                MAP_W as f32 * TILE_SIZE - VIEW_W / 2.,
                MAP_H as f32 * TILE_SIZE - VIEW_H / 2.
            )
        );
    }
}
