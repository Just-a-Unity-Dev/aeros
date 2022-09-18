use tcod::console::*;
use tcod::colors::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::BackgroundFlag::*;

mod maps;
mod tile;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 60;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r: 30, g: 30, b: 30 };
const COLOR_DARK_GROUND: Color = Color {
    r: 0,
    g: 0,
    b: 0,
};

// structs

struct Tcod {
    root: Root,
    con: Offscreen
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

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    // draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }

    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
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
    
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@',  WHITE);
    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let game = Game {
        map: maps::make_map(MAP_HEIGHT, MAP_WIDTH)
    };
    let mut objects = [player];
    
    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(LIMIT_FPS);

    while !tcod.root.window_closed() {
        // refresh the console
        tcod.con.clear();
        for object in &objects {
            object.draw(&mut tcod.con);
        }

        // render it
        render_all(&mut tcod, &game, &objects);
        tcod.root.flush();

        let player = &mut objects[0];
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
