use tcod::console::*;
use tcod::colors::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::BackgroundFlag::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

mod maps;
mod tile;

// rendering
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 60;

// map stuff
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

// FOV
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 3;

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

// structs

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap
}

struct Game {
    map: maps::Map
}

struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    /// move by the given amount
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {  
            self.x += dx;  
            self.y += dy;
        }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object], fov_recompute: bool) {
    // draw all objects in the list
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
    }
    
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[0];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // go through all tiles, and set their background color
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
        tcod.con
            .set_char_background(x, y, color, BackgroundFlag::Set);
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
    
    let player = Object::new(25, 23, '@',  WHITE);
    let game = Game {
        map: maps::make_map(MAP_HEIGHT, MAP_WIDTH)
    };
    let mut objects = [player];
    
    let mut tcod = Tcod { 
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
        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        render_all(&mut tcod, &game, &objects, fov_recompute);
        tcod.root.flush();

        let player = &mut objects[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_input(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}

fn handle_input(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key { code: Escape, .. } => return true, // exit game

        // movement keys
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),

        _ => {}
    }

    false
}
