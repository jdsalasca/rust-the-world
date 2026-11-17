use crate::actor::Wound;
use crate::game::{Game, GameState};
use crate::log::EventLog;
use macroquad::prelude::*;

fn bar(x: f32, y: f32, width: f32, height: f32, fraction: f32, color: Color) {
    draw_rectangle(x - 2.0, y - 2.0, width + 4.0, height + 4.0, Color::from_rgba(0, 0, 0, 180));
    draw_rectangle(x, y, width, height, Color::from_rgba(60, 50, 45, 255));
    draw_rectangle(x, y, width * fraction.clamp(0.0, 1.0), height, color);
}

fn wound_color(wound: Wound) -> Color {
    match wound {
        Wound::None => Color::from_rgba(79, 119, 45, 255),
        Wound::Light => Color::from_rgba(224, 164, 88, 255),
        Wound::Severe => Color::from_rgba(209, 73, 91, 255),
    }
}

fn draw_log(log: &EventLog) {
    let lines: Vec<&String> = log.lines().iter().collect();
    for (index, line) in lines.iter().enumerate() {
        let y = screen_height() - 26.0 - (lines.len() - 1 - index) as f32 * 22.0;
        draw_text(line, 22.0, y, 20.0, Color::from_rgba(235, 230, 220, 220));
    }
}

fn draw_stats(game: &Game) {
    let player = game.player();
    bar(22.0, 86.0, 240.0, 16.0, player.hp / 100.0, Color::from_rgba(190, 60, 60, 255));
    draw_text(&format!("{:.0}", player.hp.max(0.0)), 26.0, 99.0, 14.0, WHITE);

    bar(22.0, 114.0, 240.0, 12.0, player.hunger / 100.0, Color::from_rgba(220, 180, 70, 255));
    draw_text("hambre", 268.0, 124.0, 16.0, Color::from_rgba(220, 210, 190, 255));

    for (index, wound) in player.wounds.iter().enumerate() {
        let x = 22.0 + index as f32 * 22.0;
        draw_rectangle(x, 140.0, 16.0, 16.0, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle(x + 1.0, 141.0, 14.0, 14.0, wound_color(*wound));
    }
    draw_text(
        "brazos / piernas",
        118.0,
        153.0,
        16.0,
        Color::from_rgba(200, 190, 175, 255),
    );

    draw_text(
        &format!("{} monedas   {} · {} comida · {} vendas",
            player.coins, player.weapon.name(), player.food, player.bandages),
        22.0,
        178.0,
        18.0,
        Color::from_rgba(240, 220, 160, 255),
    );

    let state_text = if game.paused {
        "PAUSA".to_owned()
    } else {
        format!("x{:.0}", game.speed)
    };
    draw_text(&state_text, screen_width() - 120.0, 40.0, 26.0, WHITE);
}

fn draw_shop(game: &Game) {
    let player = game.player();
    let width = 460.0;
    let height = 230.0;
    let x = (screen_width() - width) / 2.0;
    let y = (screen_height() - height) / 2.0;

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 120),
    );
    draw_rectangle(x, y, width, height, Color::from_rgba(20, 18, 16, 255));
    draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(200, 170, 110, 255));
    draw_text("MERCADER", x + 24.0, y + 44.0, 30.0, Color::from_rgba(235, 220, 180, 255));
    draw_text(
        &format!("Tienes {} monedas", player.coins),
        x + 24.0,
        y + 78.0,
        20.0,
        GRAY,
    );
    draw_text("1", x + 24.0, y + 118.0, 20.0, WHITE);
    draw_text("Comida", x + 58.0, y + 118.0, 20.0, WHITE);
    draw_text("5", x + 380.0, y + 118.0, 20.0, Color::from_rgba(240, 220, 160, 255));
    draw_text("2", x + 24.0, y + 148.0, 20.0, WHITE);
    draw_text("Venda", x + 58.0, y + 148.0, 20.0, WHITE);
    draw_text("8", x + 380.0, y + 148.0, 20.0, Color::from_rgba(240, 220, 160, 255));
    draw_text("3", x + 24.0, y + 178.0, 20.0, WHITE);
    draw_text("Vender arma de repuesto", x + 58.0, y + 178.0, 20.0, WHITE);
    draw_text("E / Esc  Cerrar", x + 24.0, y + 212.0, 18.0, GRAY);
}

fn draw_death() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 180),
    );
    let title = "HAS MUERTO";
    let size = 48.0;
    let measure = measure_text(title, None, size as u16, 1.0);
    draw_text(
        title,
        (screen_width() - measure.width) / 2.0,
        screen_height() / 2.0 - 10.0,
        size,
        Color::from_rgba(209, 73, 91, 255),
    );
    let hint = "R para volver a empezar";
    let measure = measure_text(hint, None, 24, 1.0);
    draw_text(
        hint,
        (screen_width() - measure.width) / 2.0,
        screen_height() / 2.0 + 40.0,
        24.0,
        LIGHTGRAY,
    );
}

fn draw_controls() {
    draw_text(
        "WASD mover · clic atacar · E tienda · F comer · G vendar",
        22.0,
        40.0,
        18.0,
        Color::from_rgba(200, 190, 175, 200),
    );
    draw_text(
        "Espacio pausa · Tab velocidad · Esc salir",
        22.0,
        62.0,
        18.0,
        Color::from_rgba(160, 150, 140, 200),
    );
}

pub fn draw(game: &Game) {
    draw_controls();
    draw_stats(game);
    draw_log(&game.log);

    match game.state {
        GameState::Shop => draw_shop(game),
        GameState::Dead => draw_death(),
        GameState::Playing => {}
    }

}

