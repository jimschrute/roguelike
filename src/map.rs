use super::Position;
use super::Rect;
use rltk::{Algorithm2D, BaseMap, Console, Point, RandomNumberGenerator, Rltk, RGB};
use specs::Entity;
use std::cmp::{max, min};

pub const MAP_WIDTH: i32 = 15;
pub const MAP_HEIGHT: i32 = 15;
pub const MAX_ROOMS: usize = 2;
pub const MAP_SIZE: usize = (MAP_WIDTH * MAP_HEIGHT) as usize;
pub const MIN_ROOM_SIZE: i32 = 3;
pub const MAX_ROOM_SIZE: i32 = 6;

#[derive(Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(PartialEq, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    // fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
    //     let pos1 = self.pos_from_idx(idx1);
    //     let pos2 = self.pos_from_idx(idx2);
    //     let p1 = Point::new(pos1.x, pos1.y);
    //     let p2 = Point::new(pos2.x, pos2.y);
    //     let distance = rltk::DistanceAlg::Pythagoras.distance2d(p1, p2);
    //     println!("pathing distance is {:?}", distance);
    //     distance
    // }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        let distance = rltk::DistanceAlg::Pythagoras.distance2d(p1, p2);
        distance
    }

    fn get_available_exits(&self, idx: usize) -> Vec<(usize, f32)> {
        let avfloors1 = self.get_available_floors(idx);
        let mut exits: Vec<(usize, f32)> = Vec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }
        // println!("exits idx {} ... {:?}", idx, exits);

        if false {
            exits
        } else {
            avfloors1
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    pub fn is_inside_map(&self, x: i32, y: i32) -> bool {
        x >= 0 && x <= self.width && y >= 0 && y <= self.height
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        self.idx_from_pos(Position { x, y })
    }

    pub fn idx_from_pos(&self, pos: Position) -> usize {
        (pos.y as usize * self.width as usize) + pos.x as usize
    }

    pub fn pos_from_idx(&self, idx: usize) -> Position {
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        Position { x, y }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        let mut y = 0;
        let mut x = 0;
        for (idx, tile) in self.tiles.iter().enumerate() {
            // Render a tile depending upon the tile type
            if self.revealed_tiles[idx] {
                let (glyph, mut fg) = match tile {
                    TileType::Floor => (rltk::to_cp437('•'), RGB::from_f32(0.243, 0.537, 0.282)),
                    TileType::Wall => (rltk::to_cp437('#'), RGB::from_f32(0.451, 0.243, 0.224)),
                };
                if !self.visible_tiles[idx] {
                    fg = fg.to_greyscale()
                }
                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
            }

            // Move the coordinates
            x += 1;
            if x > self.width - 1 {
                x = 0;
                y += 1;
            }
        }
    }

    pub fn new_rooms_and_corridors(rng: &mut RandomNumberGenerator) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAP_SIZE],
            rooms: Vec::new(),
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            revealed_tiles: vec![false; MAP_SIZE],
            visible_tiles: vec![false; MAP_SIZE],
            blocked_tiles: vec![false; MAP_SIZE],
            tile_content: vec![Vec::new(); MAP_SIZE],
        };

        while map.rooms.len() < MAX_ROOMS {
            let width = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let height = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let x = rng.roll_dice(1, MAP_WIDTH - width - 1) - 1;
            let y = rng.roll_dice(1, MAP_HEIGHT - height - 1) - 1;
            let new_room = Rect::new(x, y, width, height);
            let intersects_another = map.rooms.iter().any(|room| new_room.intersect(&room));
            if !intersects_another {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let old_room = &map.rooms[map.rooms.len() - 1];
                    let Position { x: new_x, y: new_y } = new_room.center();
                    let Position {
                        x: prev_x,
                        y: prev_y,
                    } = old_room.center();
                    println!(
                        "new room: {:?}\nto old room: {:?}",
                        new_room.center(),
                        old_room.center(),
                    );
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    pub fn update_blocked_tiles(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked_tiles[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in &mut self.tile_content {
            content.clear();
        }
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn get_available_floors(&self, idx: usize) -> Vec<(usize, f32)> {
        let mut floors: Vec<(usize, f32)> = Vec::new();
        let Position { x, y } = self.pos_from_idx(idx);

        // Cardinal directions
        if self.is_floor_available(x - 1, y) {
            floors.push((self.west_idx(idx), 1.0))
        };
        if self.is_floor_available(x + 1, y) {
            floors.push((self.east_idx(idx), 1.0))
        };
        if self.is_floor_available(x, y - 1) {
            floors.push((self.north_idx(idx), 1.0))
        };
        if self.is_floor_available(x, y + 1) {
            floors.push((self.south_idx(idx), 1.0))
        };

        // Diagonals
        if self.is_floor_available(x - 1, y - 1) {
            floors.push((self.north_west_idx(idx), 1.45));
        }
        if self.is_floor_available(x + 1, y - 1) {
            floors.push((self.north_east_idx(idx), 1.45));
        }
        if self.is_floor_available(x - 1, y + 1) {
            floors.push((self.south_west_idx(idx), 1.45));
        }
        if self.is_floor_available(x + 1, y + 1) {
            floors.push((self.south_east_idx(idx), 1.45));
        }

        floors
    }

    fn north_idx(&self, idx: usize) -> usize {
        idx - self.width as usize
    }

    fn east_idx(&self, idx: usize) -> usize {
        idx + 1
    }

    fn south_idx(&self, idx: usize) -> usize {
        idx + self.width as usize
    }

    fn west_idx(&self, idx: usize) -> usize {
        idx - 1
    }

    fn south_east_idx(&self, idx: usize) -> usize {
        self.east_idx(self.south_idx(idx))
    }

    fn south_west_idx(&self, idx: usize) -> usize {
        self.west_idx(self.south_idx(idx))
    }

    fn north_east_idx(&self, idx: usize) -> usize {
        self.east_idx(self.north_idx(idx))
    }

    fn north_west_idx(&self, idx: usize) -> usize {
        self.west_idx(self.north_idx(idx))
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked_tiles[idx]
    }

    pub fn is_floor_available(&self, x: i32, y: i32) -> bool {
        self.is_inside_map(x, y) && !self.blocked_tiles[self.xy_idx(x, y)]
    }
}
