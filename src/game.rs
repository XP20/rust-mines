use rand::{distributions::Standard, prelude::Distribution, Rng};

pub enum TileVisibility {
    Visible,
    Marked,
    Hidden,
}

pub enum TileType {
    Safe,
    Mine,
}

impl Distribution<TileType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileType {
        match rng.gen_range(0..=9) {
            0..=8 => TileType::Safe,
            _ => TileType::Mine,
        }
    }
}

pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub tile_type: TileType,
    pub tile_visibility: TileVisibility,
    pub mine_count: u8,
}

impl Tile {
    fn neighbors(&self, width: usize, height: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        if self.x > 0          { neighbors.push((self.x - 1, self.y)); }  // Left
        if self.y > 0          { neighbors.push((self.x, self.y - 1)); }  // Up
        if self.x < width - 1  { neighbors.push((self.x + 1, self.y)); }  // Right
        if self.y < height - 1 { neighbors.push((self.x, self.y + 1)); }  // Down
        if self.x > 0 && self.y > 0                  { neighbors.push((self.x - 1, self.y - 1)); }  // Left Up
        if self.x > 0 && self.y < height - 1         { neighbors.push((self.x - 1, self.y + 1)); }  // Left Down
        if self.x < width - 1 && self.y > 0          { neighbors.push((self.x + 1, self.y - 1)); }  // Right Up
        if self.x < width - 1 && self.y < height - 1 { neighbors.push((self.x + 1, self.y + 1)); }  // Right Down
    
        return neighbors;
    }
}

pub struct Game {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub selected: (usize, usize),
}

impl Game {
    pub fn new(width: usize, height: usize) -> Game {
        let mut rng = rand::thread_rng();
        let mut tiles: Vec<Tile> = (0..(height*width).into()).map(|i| Tile {
            x: i % width,
            y: i / width,
            tile_type: rng.gen(),
            tile_visibility: TileVisibility::Hidden,
            mine_count: 0,
        }).collect();

        for i in 0..tiles.len() {
            let tile = &tiles[i];

            let mut mine_count: u8 = 0;
            for (x, y) in tile.neighbors(width, height).iter() {
                let neighbor_tile = &tiles[x + y * width];
                if matches!(neighbor_tile.tile_type, TileType::Mine) {
                    mine_count += 1;
                }
            }
            tiles[i].mine_count = mine_count;
        }

        Game {
            width: width,
            height: height,
            tiles: tiles,
            selected: (0, 0),
        }
    }

    pub fn end_game(&self, message: &str) {
        println!("{}", message);
    }

    pub fn check_game_won(&self) {
        if !self.tiles.iter().any(|x|
            matches!(x.tile_visibility, TileVisibility::Hidden) &&
            matches!(x.tile_type, TileType::Safe)
        ) { self.end_game("Game won ^-^"); }
    }

    pub fn toggle_mark(&mut self) {
        let (x, y) = self.selected;
        let tile = &mut self.tiles[x + y * self.width];
        let tile_visibility = &mut tile.tile_visibility;
        match tile_visibility {
            TileVisibility::Hidden => *tile_visibility = TileVisibility::Marked,
            TileVisibility::Marked => *tile_visibility = TileVisibility::Hidden,
            _ => (),
        };
        self.check_game_won();
    }

    pub fn flood_reveal(&mut self, x: usize, y: usize) {
        let tile = &mut self.tiles[x + y * self.width];
        tile.tile_visibility = TileVisibility::Visible;

        if tile.mine_count != 0 { return; }

        for (x, y) in tile.neighbors(self.width, self.height) {
            let neighbor_tile = &self.tiles[x + y * self.width];
            if matches!(neighbor_tile.tile_visibility, TileVisibility::Hidden)
            && matches!(neighbor_tile.tile_type, TileType::Safe) {
                self.flood_reveal(x, y);
            }
        }
    }

    pub fn click_tile(&mut self) {
        let (x, y) = self.selected;
        let tile = &mut self.tiles[x + y * self.width];

        if matches!(tile.tile_visibility, TileVisibility::Marked) { return; }

        tile.tile_visibility = TileVisibility::Visible;
        match tile.tile_type {
            TileType::Mine => self.end_game("You exploded >_<"),
            TileType::Safe => self.flood_reveal(x, y),
        };
        self.check_game_won();
    }

    pub fn set_selected(&mut self, pos: (i32, i32)) {
        let (x, y) = pos;
        if x >= 0 && x < self.width as i32
        && y >= 0 && y < self.height as i32 {
            self.selected = (x as usize, y as usize);
        }
    }
}
