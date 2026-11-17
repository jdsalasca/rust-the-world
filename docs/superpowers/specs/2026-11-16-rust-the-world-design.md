# rust-the-world — diseño v1

Fecha: 2026-11-16

## Intención

Primer videojuego del autor, 100% Rust. El núcleo es **Kenshi**: un mundo hostil que no gira alrededor del jugador. No es un RPG de héroe; es supervivencia emergente en una región pequeña.

La meta de v1 no es "Kenshi completo": es un bucle jugable y demostrable de 10–20 minutos, con arte pixel propio, publicado en GitHub como `rust-the-world`.

## Objetivos

1. Un personaje jugable, una región, un pueblo y un campamento de bandidos.
2. Combate cuerpo a cuerpo en tiempo real, con pausa y velocidad 1x/2x.
3. Heridas por zona (brazos y piernas), sangrado y hambre.
4. IA faccional: guardias, bandidos y alimañas se pelean entre sí sin que el jugador intervenga.
5. Saquear cuerpos, comer, vendar heridas y comerciar con el mercader.
6. Muerte permanente: al morir se reinicia la partida.
7. Arte generado con el MCP `aseprite-asset-mcp`.

## No objetivos (v1)

Reclutamiento de escuadrón, bases, crafting, árboles de diálogo, reputación de facciones, mundo procedural, monturas, guardado, audio, soporte de mando, menús de opciones, i18n.

## Anti-objetivos

- Que el jugador sea el centro del mundo (todo espera su llegada).
- Mecánicas invisibles: cada herida, hambre o cambio de estado debe verse en HUD o log.
- Grind: nada de repetir una acción 50 veces para progresar.

## Restricciones

- Rust 1.99, edición 2024, `macroquad` como única dependencia gráfica.
- Sin engine externo, sin scripting, sin assets de terceros: el arte sale del MCP de Aseprite.
- Sprites de 32×32, un set de animaciones por personaje, espejado horizontal (no hay 4 direcciones).
- Windows como plataforma objetivo.
- Repo en GitHub (`rust-the-world`), trabajo en `develop`; `main` solo recibe merges.

## Enfoques considerados

- **A. 2D cenital en tiempo real con pausa — `macroquad` (elegido).** Compilación rápida, API pequeña, el esfuerzo se va en la simulación. Contra: sin ECS, la simulación se escribe a mano con `Vec`s.
- **B. Roguelike por turnos en terminal — `ratatui`.** Máxima simulación por línea. Contra: no es el "videojuego" que se pidió.
- **C. 2D con `bevy` (ECS).** Escala bien y es idiomático para simulación. Contra: compilaciones lentas y curva de motor que se come el primer proyecto.

Se eligió **A**.

## Decisiones tomadas (revisables por el autor)

### Presentación
- Ventana 1280×720, cámara con zoom 2, tiles de 32 px.
- Cámara sigue al jugador con suavizado; sin scroll parallax.
- Sin audio en v1.

### Mundo
- Mapa de 64×48 tiles (≈3.2×4.4 pantallas a zoom 2), generado con semilla fija: pueblo al norte, campamento de bandidos al suroeste, camino que los une, rocas, cactus y un pozo.
- Colisión por tiles bloqueados (rocas, agua, paredes) más colisión circular entre actores.
- Reinicio con `R`: reconstruye el mundo entero desde la semilla (cadáveres, botín, posiciones e IA vuelven al estado inicial). Muerte permanente = la run se pierde, no se recupera nada.

### Personaje y necesidades
- Vida 100. Hambre 100, baja 1 cada 15 s; con hambre 0 se pierde vida lentamente.
- Cuatro zonas de herida: brazo izq., brazo der., pierna izq., pierna der.
- Herida leve: −15 % daño (brazos) o −15 % velocidad (piernas) por miembro. Herida grave: además, sangrado.
- Sangrado: −0.5 vida/s por herida sangrante; se detiene con venda.
- Una herida deja de penalizar a los 90 s fuera de combate; las graves dejan de sangrar con venda y también curan a los 90 s.

### Caído y muerte
- A 0 de vida el actor queda **caído** (inconsciente, en el suelo), no muerto.
- Si sigue sangrando, pierde vida hasta morir. Si se le venda (jugador, aliado o IA) o deja de sangrar, se levanta a los 15 s con 10 de vida.
- Un actor caído puede saquearse y rematarse con un ataque.
- El jugador caído no puede vendarse solo: si nadie lo levanta, muere y aparece la pantalla de fin (esto hace que la venda importe antes de caer).

### Combate
- Ataque al objetivo más cercano dentro de un arco frontal de 90°, alcance 40 px.
- Windup de 250 ms, daño 6–14 según arma, sin bloqueo ni parry.
- Al recibir daño se aplica a una zona aleatoria; probabilidad de herida grave según daño.
- Estados: en pie, caído (0 vida o piernas destrozadas), muerto.

### Inteligencia artificial
- Estados: Idle, Patrullar, Perseguir, Atacar, Huir, Caído, Muerto.
- Pathfinding A* sobre la rejilla de tiles (mapa 64×48, trivial en coste); el movimiento sigue el camino con colisión.
- Huir con vida < 25 %.
- Matriz de agresión: bandidos atacan a todos, guardias atacan a bandidos, alimañas atacan a cualquiera cercano, civiles y mercader huyen.
- **Agresión reactiva:** quien recibe daño se vuelve hostil a su agresor, incluido el jugador. Atacar a un civil o a un guardia hace que el pueblo entero lo persiga; no hay reputación persistente, solo hostilidad durante la run.

### Economía y botín
- Inventario simple por contadores: comida, vendas, monedas y arma equipada. Sin mochila con casillas.
- Al morir, un actor suelta su arma, comida, venda y monedas como objetos en el suelo; se recogen al pasar.
- Mercader: vende comida (5 monedas) y vendas (8), y compra el arma de repuesto por la mitad de su valor. Se interactúa con `E` junto a él: panel con 2 filas de compra y 1 de venta.
- Armas: puños (6), daga (8), espada (11), hacha (14), con alcance y windup distintos.

### Interfaz
- HUD: barras de vida y hambre, iconos de las cuatro heridas (sano/herido/grave), monedas.
- Log de eventos estilo Kenshi (últimas 5 líneas): "Bandido hiere tu brazo derecho", "El guardia mata al bandido".
- Pausa con `Espacio`, velocidad 1x/2x con `Tab`.
- Mensaje de muerte y reinicio con `R`.

### Arte (MCP `aseprite-asset-mcp`)
- Personajes: jugador (nómada), bandido, guardia, mercader, perro; idle/walk/attack/hurt en sheet 32×32.
- Terreno: arena, tierra, roca, camino, agua (tiles 32×32).
- Props: tienda de campaña, hoguera, caja, barril, cactus, roca, pozo.
- Iconos de objeto: comida, venda, moneda, arma.

### Verificación
- `cargo test` cubre lógica pura: daño por zona, sangrado, hambre, transición de estados de IA, economía.
- `cargo run` a 60 fps sin panics; screenshot de la partida en el README.
- Prueba manual guiada: caminar, pelear, sangrar, vendar, comer, comprar, morir, reiniciar.
- Un bandido y un guardia se pelean solos al cruzarse.

## Riesgos conocidos

- **Arte limitado:** 4 frames y una sola dirección; se compensa con espejado y animaciones cortas.
- **IA sin pathfinding:** puede quedarse trabada en esquinas; se acepta en un mapa de una pantalla y media.
- **Alcance:** cualquier idea nueva que no esté en "Objetivos" va a v2.

## Lo que queda al builder

Estructura de módulos, nombres internos, constantes exactas de balance, formato del mapa y detalles de composición del arte.
