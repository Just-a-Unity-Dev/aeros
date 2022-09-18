use crate::tile::Tile;

pub type Map = Vec<Vec<Tile>>;

pub fn make_map(map_height: i32, map_width: i32) -> Map {
    let mut map = vec![vec![Tile::empty(); map_height as usize]; map_width as usize];
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();
    map
}