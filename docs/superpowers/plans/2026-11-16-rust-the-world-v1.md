# rust-the-world v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Un juego 2D cenital en Rust puro donde eres un don nadie en un desierto hostil: pelear, sangrar, vendar, comerciar y morir, con facciones que se pelean solas.

**Architecture:** Un binario `macroquad` con módulos de dominio puros (mapa, actores, combate, IA, economía) que se testean sin ventana, y una capa fina de `main.rs` para input, cámara y dibujo. Los sistemas se comunican por structs planos y eventos de texto para el log; no hay ECS ni dependencias extra.

**Tech Stack:** Rust 1.99, edición 2024, `macroquad = "0.4"` (0.4.16), arte pixel generado con el MCP `aseprite-asset-mcp`.

**Spec:** `docs/superpowers/specs/2026-11-16-rust-the-world-design.md`

## Global Constraints

- Solo dependencia externa: `macroquad = "0.4"`. Nada de `rand`, `serde` ni assets de terceros.
- Todo el arte sale del MCP `aseprite-asset-mcp` (tools `create_canvas`, `draw_rectangle_at`, `draw_pixels_at`, `set_tag`, `export_spritesheet`, `export_frame`).
- Sprites de personaje: frames de 32×32, una sola dirección, espejado horizontal en runtime.
- Textos de UI y log en español.
- Ventana 1280×720, cámara con zoom 2, tiles de 32 px.
- 60 fps; sin panics en ninguna ruta de juego.
- Commits atómicos en la rama `develop`; `main` solo recibe merges.
- Nada de guardado: morir reinicia el mundo desde la semilla.

## Review Focus

- Atacar a un actor caído o muerto no debe paniquear ni soltar botín dos veces.
- Abrir la tienda con el mercader muerto o lejos no debe abrir el panel ni romper el bucle.
- Un destino inalcanzable para A* debe devolver `None` y dejar a la IA en `Idle`, sin bucle infinito.
- La cámara en los bordes del mapa no debe mostrar fuera del mapa ni producir UVs negativas.
- Morir con hambre y sangrado a la vez debe disparar una sola transición a `Dead` y un solo drop.

---

### Task 1: Repo, ventana y mundo visible

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `src/main.rs`, `src/world.rs`, `README.md`

**Interfaces:**
- Consumes: nada.
- Produces: `world::{TILE_SIZE: f32 = 32.0, MAP_W: i32 = 64, MAP_H: i32 = 48, Tile, Map}`, `Map::generate(seed: u64) -> Map`, `Map::tile(x, y) -> Tile`, `Map::blocked(x, y) -> bool`, `Map::blocked_at(pos: Vec2) -> bool`, `Map::clamp_camera(target: Vec2) -> Vec2`, `Map::town_spawn() -> Vec2`, `Map::camp_spawn() -> Vec2`, `Rng`.

- [ ] **Step 1: Crear el proyecto y el repo**

```bash
cargo init --name rust-the-world --edition 2024 .
git init -b main
git checkout -b develop
```

`.gitignore`: `/target` y `artifacts/`. `Cargo.toml` lleva `macroquad = "0.4"` y `[profile.dev] opt-level = 1`.

- [ ] **Step 2: Escribir los tests de `world` (fallan)**

En `src/world.rs`:
- `generation_is_deterministic`: `Map::generate(7).tiles == Map::generate(7).tiles`.
- `generation_has_all_terrain`: el mapa contiene al menos un `Road`, un `Water` y un `Rock`.
- `border_is_blocked`: `blocked(0, 0)` y `blocked(-3, 5)` son `true`.
- `camera_clamps_to_map`: `clamp_camera(vec2(-500., -500.)) == vec2(VIEW_W/2., VIEW_H/2.)` con `VIEW_W = 20. * TILE_SIZE`, `VIEW_H = 11.25 * TILE_SIZE`.

- [ ] **Step 3: Ejecutar y ver el fallo**

Run: `cargo test world::tests`
Expected: FAIL (`Map` no existe).

- [ ] **Step 4: Implementar `world.rs`**

`Tile { Sand, Dirt, Road, Rock, Water }`. `Rng` xorshift64 con `next_u32`, `range(lo, hi)`, `chance(p)`. Generación: arena base, borde de roca de 1 tile, camino en L de 3 tiles de ancho entre pueblo `(32, 12)` y campamento `(10, 38)`, plazas de tierra de 9×7, un estanque de 5×4 al este, y 40 rocas dispersas que nunca pisan camino ni plazas. `find_path` llega en Task 2: aquí solo declara `blocked`, `tile`, `blocked_at`, `clamp_camera` y los spawns.

- [ ] **Step 5: Ventana con el mapa dibujado**

`main.rs`: `#[macroquad::main("rust-the-world")]`, cámara `zoom = vec2(2., 2.)` siguiendo a un `Vec2` de prueba con `clamp_camera`, dibuja cada tile como rectángulo de color (Sand `#c2a15a`, Dirt `#8a6f47`, Road `#b08d5a`, Rock `#4a4a55`, Water `#3b6ea5`). Sin actores todavía.

- [ ] **Step 6: Verificar**

Run: `cargo test && cargo run`
Expected: tests verdes; ventana con el desierto, camino y estanque; `Esc` cierra.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml .gitignore src README.md
git commit -m "feat: ventana y mapa del desierto"
```

---

### Task 2: Movimiento y pathfinding

**Files:**
- Modify: `src/world.rs`, `src/main.rs`

**Interfaces:**
- Produces: `Map::find_path(from: IVec2, to: IVec2) -> Option<Vec<IVec2>>` (A* 4-vecinos, sin la casilla inicial), `Map::line_blocked(a: Vec2, b: Vec2) -> bool`.

- [ ] **Step 1: Tests (fallan)**

- `path_goes_around_wall`: mapa 5×5 construido a mano con una pared vertical salvo un hueco; el camino no contiene ninguna celda bloqueada y empieza y termina donde se pide.
- `path_to_blocked_is_none`: destino sobre roca devuelve `None`.
- `path_unreachable_is_none`: destino encerrado por roca devuelve `None`.
- `line_blocked_detects_wall`: segmento que cruza una pared devuelve `true`.

Para construir mapas de test, añade `Map::from_rows(rows: &[&str]) -> Map` (`#` roca, `.` arena, `=` camino).

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test world::tests`
Expected: FAIL.

- [ ] **Step 3: Implementar A***

Abiertos por `Vec` con coste `g` y heurística Manhattan (mapa de 3072 celdas: no hace falta heap). Reconstruye el camino desde el padre. `find_path` devuelve `None` si `from == to`, si alguno está bloqueado o si no hay ruta.

- [ ] **Step 4: Movimiento del jugador**

WASD/ flechas a 140 px/s con `Map::blocked_at` por eje (deslizar contra paredes). Sin actores aún.

- [ ] **Step 5: Verificar**

Run: `cargo test && cargo run`
Expected: tests verdes; el jugador (rectángulo claro) se mueve y no atraviesa roca ni agua.

- [ ] **Step 6: Commit**

```bash
git commit -am "feat: A* y movimiento con colisión"
```

---

### Task 3: Actores, heridas y necesidades

**Files:**
- Create: `src/actor.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `Faction { Player, Town, Bandit, Wildlife }`, `Role { Player, Guard, Merchant, Civilian, Bandit, Dog }`, `Limb { ArmL, ArmR, LegL, LegR }` (índices 0–3), `Wound { None, Light, Severe }`, `Life { Up, Downed, Dead }`, `AiState { Idle, Patrol, Chase, Attack, Flee }`, `Weapon { Fists, Dagger, Sword, Axe }` con `damage()` y `reach()`, y `Actor`.

```rust
pub struct Actor {
    pub id: usize, pub pos: Vec2, pub vel: Vec2,
    pub faction: Faction, pub role: Role,
    pub hp: f32, pub hunger: f32,
    pub wounds: [Wound; 4], pub life: Life, pub downed: f32,
    pub ai: AiState, pub target: Option<usize>, pub hostile_to_player: bool,
    pub path: Vec<IVec2>, pub attack_cd: f32, pub facing_right: bool,
    pub food: u32, pub bandages: u32, pub coins: u32,
    pub weapon: Weapon, pub spare: Option<Weapon>,
    pub heal: [f32; 4], pub death_handled: bool,
}
```

- [ ] **Step 1: Tests (fallan)**

- `hunger_drops_one_per_fifteen_seconds`: `actor.tick(15.0)` deja `hunger` en `99.0` (±0.01).
- `starvation_damages`: hambre 0 durante 10 s quita 10 de vida.
- `leg_wound_slows`: herida `Light` en `LegL` deja `speed()` en `0.85 * base`.
- `arm_wound_weakens`: herida `Light` en `ArmR` deja `damage()` en `0.85 * base`.
- `bandage_stops_bleeding`: con 1 venda y una herida `Severe`, `bandage()` devuelve `true`, consume la venda, `is_bleeding()` pasa a `false`.
- `wound_heals_after_ninety_seconds`: herida `Severe` vendada → tras `tick(90.0)` es `None`.
- `zero_hp_goes_downed_not_dead`: `hurt(200.)` deja `Life::Downed`.
- `downed_npc_wakes`: un NPC caído sin sangrado, `tick(15.0)` → `Up` con 10 de vida.
- `downed_player_does_not_wake`: el jugador caído sin sangrado muere a los 15 s (no tiene aliados que lo levanten).
- `downed_bleeding_dies`: caído con herida severa sin vendar → tras 40 s `Life::Dead`.

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test actor::tests`
Expected: FAIL.

- [ ] **Step 3: Implementar `actor.rs`**

Constantes: `MAX_HP 100`, `MAX_HUNGER 100`, `HUNGER_PER_SEC 1.0/15.0`, `HEAL_SECS 90.0`, `WAKE_SECS 15.0`, `EXECUTE_AFTER 20.0` (vida negativa que mata a un caído). `tick(dt)` devuelve `Vec<String>` con eventos (`"Tienes hambre"`, `"Te desangras"`). Al morir, `death_handled` evita repetir la transición. `hurt(amount, rng)` elige zona al azar, marca `Severe` si `amount > 20`, y solo actúa sobre `Life::Up`.

- [ ] **Step 4: Spawns y dibujo**

Pueblo: jugador en `town_spawn()`, mercader, 2 guardias y 2 civiles alrededor. Campamento: 4 bandidos. 3 perros cerca de las rocas. Dibujo provisional: círculos de color por facción. El jugador se mueve; los demás están quietos.

- [ ] **Step 5: Verificar**

Run: `cargo test && cargo run`
Expected: tests verdes; ~13 figuras repartidas entre pueblo y campamento.

- [ ] **Step 6: Commit**

```bash
git add src/actor.rs src/main.rs
git commit -m "feat: actores con heridas, hambre y caída"
```

---

### Task 4: Combate y botín

**Files:**
- Create: `src/combat.rs`, `src/economy.rs`
- Modify: `src/main.rs`, `src/actor.rs`

**Interfaces:**
- Produces: `combat::resolve_hit(attacker: &Actor, target: &mut Actor, rng: &mut Rng) -> Option<Hit>`, `combat::Hit { limb: Limb, damage: f32 }`, `economy::Item { Food, Bandage, Coin(u32), Weapon(Weapon) }`, `economy::Drop { pos: Vec2, item: Item }`, `economy::pick_up(actor: &mut Actor, drops: &mut Vec<Drop>) -> Vec<String>`, `economy::drop_loot(actor: &mut Actor) -> Vec<Drop>` (marca `death_handled` para no soltar dos veces), `FOOD_PRICE 5`, `BANDAGE_PRICE 8`, `buy_food(&mut Actor) -> bool`, `buy_bandage(&mut Actor) -> bool`, `sell_spare(&mut Actor) -> bool`.

- [ ] **Step 1: Tests (fallan)**

- `out_of_reach_misses`: separación 80 px → `None`.
- `hit_damages_and_wounds`: a 20 px con `Weapon::Sword` quita 11 de vida y devuelve una `Limb`.
- `hit_on_downed_executes`: objetivo caído recibe un golpe → `Life::Dead` y suelta botín una sola vez.
- `loot_drops_once`: llamar `drop_loot` dos veces sobre un muerto devuelve vacío la segunda.
- `pickup_adds_items`: pasar sobre un drop de comida y monedas actualiza contadores y vacía el suelo.
- `buy_food_needs_coins`: con 5 monedas compra; con 4 no.
- `sell_spare_pays_half`: espada de repuesto (11 de daño) → 5 monedas.

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test combat::tests economy::tests`
Expected: FAIL.

- [ ] **Step 3: Implementar**

`resolve_hit`: exige `attacker.life == Up`, distancia ≤ `attacker.weapon.reach()` y `attack_cd <= 0`; aplica `attacker.damage()`, asigna zona al azar y pone `attack_cd` a `0.8` (espada) o `0.5` (puños). Un objetivo `Downed` muere directo (`EXECUTE`). `drop_loot` solo para `Dead` y una vez; bandido suelta 2–6 monedas, 50 % comida, 30 % venda, y su arma. `pick_up` recoge drops a menos de 16 px.

- [ ] **Step 4: Input de ataque**

Clic izquierdo → `resolve_hit` contra el actor más cercano dentro del arco frontal de 90° y de `reach()`; el log muestra `"Golpeas al bandido (pierna izquierda)"` o el fallo. Muerte → drops al suelo y log `"El guardia mata al bandido"`.

- [ ] **Step 5: Verificar**

Run: `cargo test && cargo run`
Expected: tests verdes; al golpear a un bandido baja su vida y al morir suelta monedas recogibles.

- [ ] **Step 6: Commit**

```bash
git add src/combat.rs src/economy.rs src/actor.rs src/main.rs
git commit -m "feat: combate, heridas y botín"
```

---

### Task 5: IA faccional

**Files:**
- Create: `src/ai.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `ai::hostile(a: &Actor, b: &Actor) -> bool`, `ai::pick_target(index: usize, actors: &[Actor]) -> Option<usize>`, `ai::tick(actors: &mut [Actor], map: &Map, dt: f32) -> Vec<String>`.

- [ ] **Step 1: Tests (fallan)**

- `guard_hates_bandit`, `bandit_hates_guard`, `town_is_neutral_to_player`.
- `town_turns_on_player_after_attack`: `player.hostile_to_player`… se marca `guard.hostile_to_player = true` y `hostile(&guard, &player)` es `true`.
- `civilian_flees_from_bandit`: civil a 120 px de un bandido → `AiState::Flee` y se aleja.
- `dog_attacks_player_in_range`: perro a 100 px → `Chase`.
- `unreachable_target_stays_idle`: jugador en una isla de roca → el bandido queda `Idle`, sin bucles.
- `bandit_approaches_target`: con camino libre, en 2 s la distancia baja.

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test ai::tests`
Expected: FAIL.

- [ ] **Step 3: Implementar**

Matriz: `Bandit` hostil a todo menos bandidos; `Wildlife` a todo menos fauna; `Town` a bandidos; `Player` a bandidos y fauna. Reactiva: al recibir daño, el agresor se vuelve objetivo y, si la víctima es del pueblo, todos los del pueblo marcan `hostile_to_player` si el agresor es el jugador. Roles: guardia patrulla y persigue; civil huye; mercader huye; perro persigue; bandido patrulla el campamento y persigue. `tick` recalcula objetivo cada 0.5 s, pide camino a `find_path` y mueve a `speed()`. Huir = dirección opuesta al objetivo durante 2 s.

- [ ] **Step 4: Conectar el daño con la IA**

`resolve_hit` marca `target.hostile_to_player` y el agresor como objetivo de la víctima y de sus aliados del pueblo.

- [ ] **Step 5: Verificar**

Run: `cargo test && cargo run`
Expected: tests verdes; al acercarte al campamento los bandidos te persiguen; si te alejas, vuelven.

- [ ] **Step 6: Commit**

```bash
git add src/ai.rs src/main.rs src/combat.rs
git commit -m "feat: IA faccional con agresión reactiva"
```

---

### Task 6: Arte con el MCP de Aseprite

**Files:**
- Create: `tools/paint_art.mjs`, `assets/art/**` (24 PNG), `src/render.rs`
- Modify: `src/main.rs`, `Cargo.toml`

**Interfaces:**
- Produces: `render::Art::load() -> Art`, `render::CharSheet { idle: Texture2D, walk: Texture2D, attack: Texture2D }`, `render::AnimKind { Idle, Walk, Attack }`, `render::draw_actor(sheet: &CharSheet, anim: AnimKind, t: f32, pos: Vec2, facing_right: bool)`, `render::draw_tile(tile: Tile, at: Vec2)`.
- Consumes: `Role`, `Tile`.

- [ ] **Step 1: Generador de arte**

`tools/paint_art.mjs` habla JSON-RPC (una línea por mensaje) con `node <mcp>/dist/interfaces/mcp-server.js`, `ASEPRITE_PATH` desde el entorno. Declara poses como listas de rectángulos y píxeles por frame y llama `create_canvas`, `add_frames`, `set_tag`, `draw_rectangle_at`, `draw_pixels_at` y `export_spritesheet`/`export_frame`. Nombres exactos:

- `assets/art/chars/{player,bandit,guard,merchant,dog}_{idle,walk,attack}.png` — frames 32×32; idle 2, walk 4, attack 3.
- `assets/art/tiles/{sand,dirt,road,rock,water}.png` — 32×32.
- `assets/art/props/{cactus,rock,tent,fire,crate,well}.png` — 32×32.
- `assets/art/items/{food,bandage,coin,sword}.png` — 16×16.

- [ ] **Step 2: Generar y validar**

Run: `node tools/paint_art.mjs`
Expected: 24 PNG; `chars/player_idle.png` mide 64×32, `player_walk.png` 128×32, `player_attack.png` 96×32, tiles 32×32, items 16×16. Abrir el sheet del jugador y confirmar que se lee como una persona.

- [ ] **Step 3: Cargar arte embebido**

`render.rs` con `include_bytes!` + `Texture2D::from_file_with_format` y `FilterMode::Nearest`. `Art::load()` construye los 5 `CharSheet`, los 5 tiles, 6 props y 4 iconos. Sustituir los rectángulos de `main.rs` por `draw_tile`/`draw_actor`; props en su posición del mundo; items en el suelo.

- [ ] **Step 4: Verificar**

Run: `cargo build && cargo run`
Expected: el desierto con texturas, personajes animados (idle al estar quieto, walk al moverse, attack al golpear) y espejado al cambiar de dirección. Screenshot de la ventana guardado como `docs/media/gameplay.png`.

- [ ] **Step 5: Commit**

```bash
git add tools/paint_art.mjs assets src/render.rs src/main.rs Cargo.toml docs/media
git commit -m "feat: arte pixel generado con el MCP de Aseprite"
```

---

### Task 7: HUD, log, comercio, pausa y muerte

**Files:**
- Create: `src/hud.rs`, `src/log.rs`
- Modify: `src/main.rs`, `src/economy.rs`

**Interfaces:**
- Produces: `EventLog::new()`, `EventLog::push(String)`, `EventLog::lines() -> &VecDeque<String>` (máximo 5), `GameState { Playing, Shop, Dead }`, `Game { map: Map, actors: Vec<Actor>, drops: Vec<Drop>, log: EventLog, state: GameState, speed: f32, rng: Rng, player: usize }`, `Game::new(seed: u64) -> Game`, `Game::restart(&mut self)`, `economy::can_trade(player: &Actor, merchant: &Actor) -> bool`, `economy::eat(&mut Actor) -> bool`, `hud::draw(game: &Game, art: &Art)`, `hud::draw_shop(player: &Actor)`.

- [ ] **Step 1: Tests (fallan)**

- `log_keeps_last_five`.
- `shop_requires_live_merchant_and_range`: `can_trade(player, merchant)` falso a 200 px y falso con mercader `Dead`.
- `eat_restores_hunger`: `eat(&mut actor)` con 1 comida sube hambre en 40 y consume la comida.
- `restart_resets_world`: `Game::restart()` produce el mismo mapa y los mismos 13 actores que el inicio.

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test hud::tests economy::tests`
Expected: FAIL.

- [ ] **Step 3: Implementar HUD y estados**

Barras de vida (roja) y hambre (amarilla); 4 iconos de herida (verde/amarillo/rojo); monedas; velocidad (`1x`/`2x`); log abajo a la izquierda con las 5 últimas líneas. Controles: `Espacio` pausa, `Tab` alterna 1x/2x, `E` tienda junto al mercader, `F` come, `G` venda, `R` reinicia, `Esc` cierra. Tienda: `1` comida, `2` venda, `3` vender repuesto, `E`/`Esc` cerrar; el mundo se detiene mientras está abierta. Muerte: pantalla oscura con `"HAS MUERTO"` y `"R para volver a empezar"`.

- [ ] **Step 4: Verificar**

Run: `cargo test && cargo run`
Expected: tests verdes; comer sube el hambre, la tienda descuenta monedas, morir congela el mundo y `R` reinicia.

- [ ] **Step 5: Commit**

```bash
git add src/hud.rs src/log.rs src/main.rs src/economy.rs
git commit -m "feat: HUD, comercio, pausa y muerte"
```

---

### Task 8: Cierre, README y GitHub

**Files:**
- Modify: `README.md`, `docs/superpowers/specs/2026-11-16-rust-the-world-design.md`
- Create: `docs/media/gameplay.png`

**Interfaces:**
- Consumes: todo lo anterior.

- [ ] **Step 1: Prueba manual guiada**

Run: `cargo run`
Camina al campamento, pelea, sangra, venda, come, compra, muere y reinicia. Confirmar que un bandido y un guardia se pelean solos al cruzarse.

- [ ] **Step 2: Calidad**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
Expected: sin cambios de formato, sin warnings, tests verdes.

- [ ] **Step 3: README**

Controles, bucle de juego, cómo regenerar el arte (`ASEPRITE_PATH=... node tools/paint_art.mjs`), screenshot y crédito al MCP.

- [ ] **Step 4: Commit y repo remoto**

```bash
git add README.md docs
git commit -m "docs: README con controles y captura"
gh repo create rust-the-world --public --source . --remote origin --push
git checkout main && git merge develop && git push origin main && git checkout develop
```

- [ ] **Step 5: Verificar el remoto**

Run: `gh repo view --web`
Expected: repo público con `main` y `develop`, README visible y screenshot cargando.
