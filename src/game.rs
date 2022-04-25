use chrono::{DateTime, Utc};
use dashmap::DashMap;
use num_integer::Roots;
use rand::Rng;
use serenity::prelude::TypeMapKey;
use std::{sync::Arc, time::SystemTime};

pub struct Tile {
    pub is_mine: bool,
    pub is_flagged: bool,
    pub is_revealed: bool,
    pub adjacent_mines: u32,
}

impl Tile {
    pub fn new() -> Self {
        Tile {
            is_mine: false,
            is_flagged: false,
            is_revealed: false,
            adjacent_mines: 0,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum GameState {
    Won,
    Playing,
    Lost,
    NotStarted,
}

pub struct Game {
    pub height: u32,
    pub width: u32,
    pub tiles: Vec<Vec<Tile>>,
    pub number_of_mines: u32,
    pub unmined_tiles: u32,
    pub placed_flag_count: u32,
    pub state: GameState,
    pub time_started: DateTime<Utc>,
    pub last_move_time: DateTime<Utc>,
}

impl Game {
    pub fn new(width: u32, height: u32, number_of_mines: u32) -> Self {
        Game {
            height,
            width,
            tiles: (0..height)
                .map(|_| (0..width).map(|_| Tile::new()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            number_of_mines,
            unmined_tiles: width * height,
            placed_flag_count: 0,
            state: GameState::NotStarted,
            time_started: DateTime::<Utc>::from(SystemTime::now()),
            last_move_time: DateTime::<Utc>::from(SystemTime::now()),
        }
    }

    fn is_out_of_bounds(&self, position: (i32, i32)) -> bool {
        if position.0 < 0 || position.0 as i32 > self.width as i32 - 1 {
            return true;
        }
        if position.1 < 0 || position.1 as i32 > self.height as i32 - 1 {
            return true;
        }
        return false;
    }

    fn make_adjacent_tiles_visible(&mut self, position: (i32, i32)) {
        if self.is_out_of_bounds((position.0, position.1)) {
            return;
        }

        let tile = &mut self.tiles[position.1 as usize][position.0 as usize];

        if tile.is_revealed || tile.is_mine {
            return;
        }

        tile.is_revealed = true;
        self.unmined_tiles -= 1;

        if tile.is_flagged {
            tile.is_flagged = false;
            self.placed_flag_count -= 1;
        }

        if tile.adjacent_mines != 0 {
            return;
        }

        self.make_adjacent_tiles_visible((position.0 - 1, position.1));
        self.make_adjacent_tiles_visible((position.0 + 1, position.1));
        self.make_adjacent_tiles_visible((position.0 - 1, position.1 - 1));
        self.make_adjacent_tiles_visible((position.0 - 1, position.1 + 1));
        self.make_adjacent_tiles_visible((position.0 + 1, position.1 - 1));
        self.make_adjacent_tiles_visible((position.0 + 1, position.1 + 1));
        self.make_adjacent_tiles_visible((position.0, position.1 - 1));
        self.make_adjacent_tiles_visible((position.0, position.1 + 1));
    }

    fn can_place_mine(&self, position: (u32, u32), dug_position: (u32, u32)) -> bool {
        if ((dug_position.0 as i32 - position.0 as i32)
            * (dug_position.0 as i32 - position.0 as i32)
            + (dug_position.1 as i32 - position.1 as i32)
                * (dug_position.1 as i32 - position.1 as i32))
            .sqrt()
            < 3
        {
            return false;
        }

        if self.tiles[position.1 as usize][position.0 as usize].is_mine {
            false
        } else {
            true
        }
    }

    fn generate_mines(&mut self, position: (u32, u32)) {
        for _ in 0..self.number_of_mines {
            let mut random_position = (
                rand::thread_rng().gen_range(0..self.width),
                rand::thread_rng().gen_range(0..self.height),
            );

            while !self.can_place_mine(random_position, position) {
                random_position = (
                    rand::thread_rng().gen_range(0..self.width),
                    rand::thread_rng().gen_range(0..self.height),
                );
            }

            self.tiles[random_position.1 as usize][random_position.0 as usize].is_mine = true;
        }

        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut limit_x_lower = -1;
                let mut limit_x_upper = 1;

                let mut limit_y_lower = -1;
                let mut limit_y_upper = 1;

                if x == 0 {
                    limit_x_lower = 0;
                }
                if x == self.width as i32 - 1 {
                    limit_x_upper = 0;
                }

                if y == 0 {
                    limit_y_lower = 0;
                }
                if y == self.height as i32 - 1 {
                    limit_y_upper = 0;
                }

                let mut adjacent_mines = 0;

                for y2 in limit_y_lower..=limit_y_upper {
                    for x2 in limit_x_lower..=limit_x_upper {
                        if self.tiles[(y2 + y) as usize][(x2 + x) as usize].is_mine {
                            adjacent_mines += 1;
                        }
                    }
                }
                self.tiles[y as usize][x as usize].adjacent_mines = adjacent_mines;
            }
        }
    }

    fn start_dig(&mut self, position: (u32, u32)) {
        self.time_started = DateTime::<Utc>::from(SystemTime::now());
        self.last_move_time = self.time_started.clone();
        self.generate_mines(position);
        self.make_adjacent_tiles_visible((position.0 as i32, position.1 as i32));
        self.state = GameState::Playing;

        if self.unmined_tiles == self.number_of_mines {
            self.state = GameState::Won;
        }
    }

    fn single_dig(&mut self, position: (u32, u32)) {
        self.last_move_time = DateTime::<Utc>::from(SystemTime::now());
        let tile = &mut self.tiles[position.1 as usize][position.0 as usize];

        if tile.is_flagged {
            return;
        }

        if tile.is_mine {
            tile.is_revealed = true;
            self.state = GameState::Lost;
            return;
        }

        self.make_adjacent_tiles_visible((position.0 as i32, position.1 as i32));

        if self.unmined_tiles == self.number_of_mines {
            self.state = GameState::Won;
        }
    }

    pub fn dig(&mut self, position: (u32, u32)) {
        match &self.state {
            GameState::NotStarted => self.start_dig(position),
            GameState::Playing => self.single_dig(position),
            _ => (),
        }
    }

    pub fn flag(&mut self, position: (u32, u32)) {
        if self.state != GameState::Playing {
            return;
        }

        self.last_move_time = DateTime::<Utc>::from(SystemTime::now());
        let tile = &mut self.tiles[position.1 as usize][position.0 as usize];

        if !tile.is_revealed && !tile.is_flagged && self.placed_flag_count < self.number_of_mines {
            tile.is_flagged = true;
            self.placed_flag_count += 1;
        }
    }

    pub fn unflag(&mut self, position: (u32, u32)) {
        if self.state != GameState::Playing {
            return;
        }

        self.last_move_time = DateTime::<Utc>::from(SystemTime::now());
        let tile = &mut self.tiles[position.1 as usize][position.0 as usize];

        if tile.is_flagged {
            tile.is_flagged = false;
            self.placed_flag_count -= 1
        }
    }
}

pub struct GameDataKey;

impl TypeMapKey for GameDataKey {
    type Value = Arc<DashMap<u64, Game>>;
}
