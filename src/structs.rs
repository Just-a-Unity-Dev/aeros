use tcod::map::{FovAlgorithm, Map as FovMap};
use tcod::console::*;

pub struct Tcod {
    pub root: Root,
    pub con: Offscreen,
    pub fov: FovMap
}