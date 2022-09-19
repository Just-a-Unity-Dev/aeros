use tcod::console::*;
use tcod::colors::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

mod g_input;
mod structs;
mod object;
mod maps;
mod tile;
mod ai;

// rendering
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

const LIMIT_FPS: i32 = 60;
const PLAYER: usize = 0;

// map stuff
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;

// FOV
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const LANTERN_RADIUS: i32 = 50;

// colors
const COLOR_DARK_WALL: Color = Color { r: 30, g: 30, b: 30 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 90,
    g: 90,
    b: 90,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 10,
    g: 10,
    b: 10,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 180,
    g: 180,
    b: 180,
};

fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color,
) {
    // render a bar (HP, experience, etc). First calculate the width of the bar
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

    // render the background first
    panel.set_default_background(back_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    // now render the bar on top
    panel.set_default_background(bar_color);
    if bar_width > 0 {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    // finally, some centered text with the values
    panel.set_default_foreground(WHITE);
    panel.print_ex(
        x + total_width / 2,
        y,
        BackgroundFlag::None,
        TextAlignment::Center,
        &format!("{}: {}/{}", name, value, maximum),
    );
}

fn render_all(tcod: &mut structs::Tcod, game: &mut structs::Game, objects: &[object::Object], fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, LANTERN_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                // outside of field of view:
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };

            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                // since it's visible, explore it
                *explored = true;
            }
            if *explored {
                // show explored tiles only (any visible tile is explored already)
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
        .collect();
    // sort so that non-blocknig objects come first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }

    // prepare to render the GUI panel
    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();

    // show the player's stats
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARKER_RED,
    );

    // print the game messages, one line at a time
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // blit the contents of `panel` to the root console
    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, PANEL_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );


    // blit the contents of "con" to the root console
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn main() {
    let root = Root::initializer()
    .font("dejavu16x16_gs_tc.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("Aeros")
    .init();
    
    let mut player = object::Object::new(25, 23, '@', "You", DESATURATED_GREEN, false);
    player.fighter = Some(object::Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        on_death: object::DeathCallback::Player,
    });
    player.alive = true;
    player.attack_message = "attack".into();

    let mut objects = vec![player];
    let mut game = structs::Game {
        map: maps::make_map(MAP_HEIGHT, MAP_WIDTH, &mut objects),
        messages: structs::Messages::new()
    };

    game.messages.add(
        "Welcome to space. Wear your hardsuit and prepare some medicine.",
        RED,
    );

    let mut tcod = structs::Tcod { 
        root, 
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT), 
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };
    tcod::system::set_fps(LIMIT_FPS);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH { 
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        // refresh the console
        tcod.con.clear();

        // render it
        let fov_recompute = previous_player_position != (objects[PLAYER].pos());
        render_all(&mut tcod, &mut game, &objects, fov_recompute);
        tcod.root.flush();

        previous_player_position = objects[PLAYER].pos();
        let player_action = g_input::handle_input(&mut tcod, &mut game, &mut objects);

        if player_action == g_input::PlayerAction::Exit {
            break;
        }

        // let monsters take their turn
        if objects[PLAYER].alive && player_action != g_input::PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai::ai_take_turn(id, &tcod, &mut game, &mut objects);
                }
            }
        }
    }
}
