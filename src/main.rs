use tcod::console::*;
use tcod::colors::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

mod structs;
mod object;
mod input;
mod maps;
mod tile;
mod game;

// rendering
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 60;
const PLAYER: usize = 0;

// map stuff
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

// FOV
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const LANTERN_RADIUS: i32 = 5;

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

fn render_all(tcod: &mut structs::Tcod, game: &mut game::Game, objects: &[object::Object], fov_recompute: bool) {
    // draw all objects in the list
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
    }

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

    blit(&tcod.con, (0,0), (MAP_WIDTH, MAP_HEIGHT), &mut tcod.root, (0,0), 1.0, 1.0);
}

fn main() {
    let root = Root::initializer()
    .font("dejavu16x16_gs_tc.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("Aeros")
    .init();
    
    let mut player = object::Object::new(25, 23, '@', "player", WHITE, false);
    player.alive = true;

    let mut objects = vec![player];
    let mut game = game::Game {
        map: maps::make_map(MAP_HEIGHT, MAP_WIDTH, &mut objects)
    };

    let mut tcod = structs::Tcod { 
        root, 
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT), 
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT) 
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
        for object in &objects {
            object.draw(&mut tcod.con);
        }

        // render it
        let fov_recompute = previous_player_position != (objects[PLAYER].pos());
        render_all(&mut tcod, &mut game, &objects, fov_recompute);
        tcod.root.flush();

        previous_player_position = objects[PLAYER].pos();
        let player_action = input::handle_input(&mut tcod, &game, &mut objects);

        // let monsters take their turn
        if objects[PLAYER].alive && player_action != input::PlayerAction::DidntTakeTurn {
            for object in &objects {
                // only if object is not player
                if (object as *const _) != (&objects[PLAYER] as *const _) {
                    println!("The {} growls!", object.name);
                }
            }
        }

        if player_action == input::PlayerAction::Exit {
            break;
        }
    }
}
