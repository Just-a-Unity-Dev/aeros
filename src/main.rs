use tcod::console::*;
use tcod::colors::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::BackgroundFlag::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 60;

struct Tcod {
    root: Root
}

fn main() {
    let root = Root::initializer()
        .font("dejavu16x16_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Aeros")
        .init();
    
    let mut player_x: i32 = SCREEN_WIDTH / 2;
    let mut player_y: i32 = SCREEN_HEIGHT / 2;
    
    let mut tcod = Tcod { root };
    tcod::system::set_fps(LIMIT_FPS);
    while !tcod.root.window_closed() {
        tcod.root.set_default_foreground(WHITE);
        tcod.root.clear();
        tcod.root.put_char(player_x, player_y, '@', None);
        tcod.root.flush();

        let exit = handle_input(&mut tcod, &mut player_x, &mut player_y);
        if exit {
            break;
        }
    }
}

fn handle_input(tcod: &mut Tcod, player_x: &mut i32, player_y: &mut i32) -> bool {
    // TODO: handle input
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key { code: Escape, .. } => return true, // exit game

        // movement keys
        Key { code: Up, .. } => *player_y -= 1,
        Key { code: Down, .. } => *player_y += 1,
        Key { code: Left, .. } => *player_x -= 1,
        Key { code: Right, .. } => *player_x += 1,

        _ => {}
    }

    false
}
