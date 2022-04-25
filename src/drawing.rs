use std::fs::File;
use std::io::Read;

use crate::game::{Game, GameState};
use crate::text;

use rand::Rng;
use tiny_skia::*;

const LINE_WIDTH: f32 = 8.0;
const LINE_WIDTH_HALF: f32 = LINE_WIDTH / 2.0;

const NUMBER_COLORS: [(u8, u8, u8); 8] = [
    (25, 118, 210),
    (56, 142, 60),
    (211, 47, 47),
    (123, 31, 162),
    (255, 143, 0),
    (0, 128, 130),
    (0, 0, 0),
    (128, 128, 128),
];

lazy_static! {
    static ref GRASS_COLOR_DARK: Color = Color::from_rgba8(162, 209, 73, 255);
    static ref GRASS_COLOR_LIGHT: Color = Color::from_rgba8(170, 215, 81, 255);
    static ref GROUND_COLOR_DARK: Color = Color::from_rgba8(215, 184, 153, 255);
    static ref GROUND_COLOR_LIGHT: Color = Color::from_rgba8(229, 194, 159, 255);
    static ref WATER_COLOR_DARK: Color = Color::from_rgba8(148, 196, 243, 255);
    static ref WATER_COLOR_LIGHT: Color = Color::from_rgba8(153, 198, 244, 255);
    static ref BORDER_COLOR_DARK: Color = Color::from_rgba8(208, 208, 208, 255);
    static ref BORDER_COLOR_LIGHT: Color = Color::from_rgba8(220, 220, 220, 255);
    static ref GRASS_OUTLINE_COLOR: Color = Color::from_rgba8(135, 175, 58, 255);
    static ref TOP_BAR_COLOR: Color = Color::from_rgba8(74, 117, 44, 255);
    static ref FLAG_PIXMAP: Pixmap = Pixmap::decode_png({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("images");
        path.push("Flag.png");

        let file = File::open(path).unwrap();
        &file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
    static ref WARNING_PIXMAP: Pixmap = Pixmap::decode_png({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("images");
        path.push("Warning.png");

        let file = File::open(path).unwrap();
        &file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
    static ref X_MARK_PIXMAP: Pixmap = Pixmap::decode_png({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("images");
        path.push("XMark.png");

        let file = File::open(path).unwrap();
        &file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
    static ref EXPLOSION_PIXMAP: Pixmap = Pixmap::decode_png({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("images");
        path.push("Explosion.png");

        let file = File::open(path).unwrap();
        &file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
    static ref CLOCK_PIXMAP: Pixmap = Pixmap::decode_png({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("images");
        path.push("Clock.png");

        let file = File::open(path).unwrap();
        &file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
    static ref FLOWER_PIXMAPS: Vec<Pixmap> = (1..=14)
        .map(|num| {
            let mut path = std::env::current_exe().unwrap();
            path.pop();
            path.push("assets");
            path.push("images");
            path.push("flowers");
            path.push(format!("Flower{}.png", num));

            let file = File::open(path).unwrap();
            Pixmap::decode_png(&file.bytes().flatten().collect::<Vec<_>>()).unwrap()
        })
        .collect::<Vec<_>>();
}

#[derive(PartialEq)]
enum LineType {
    Vertical,
    Horizontal,
}

pub trait MinesweeperDrawer {
    fn draw_board(game: &Game) -> Pixmap;
}

pub struct DefaultMinesweeperDrawer;

impl DefaultMinesweeperDrawer {
    fn draw_line(
        map: &mut Pixmap,
        position: (f32, f32),
        length: f32,
        color: &Color,
        line_type: LineType,
    ) {
        let rect = Rect::from_xywh(
            position.0
                - (if line_type == LineType::Vertical {
                    LINE_WIDTH_HALF
                } else {
                    0.0
                }),
            position.1
                - (if line_type == LineType::Horizontal {
                    LINE_WIDTH_HALF
                } else {
                    0.0
                }),
            if line_type == LineType::Vertical {
                LINE_WIDTH
            } else {
                length
            },
            if line_type == LineType::Horizontal {
                LINE_WIDTH
            } else {
                length
            },
        )
        .unwrap();

        let paint = create_default_paint(*color);

        map.fill_rect(rect, &paint, Transform::identity(), None);
    }

    fn add_border_line(map: &mut Pixmap, game: &Game) {
        Self::draw_line(
            map,
            (100.0 - LINE_WIDTH, 100.0 - LINE_WIDTH_HALF),
            (game.width * 100) as f32 + LINE_WIDTH,
            &Color::BLACK,
            LineType::Horizontal,
        );

        Self::draw_line(
            map,
            (100.0 - LINE_WIDTH_HALF, 100.0),
            (game.height * 100) as f32,
            &Color::BLACK,
            LineType::Vertical,
        );
    }

    fn draw_icon(position: (i32, i32), map: &mut Pixmap, icon_map: PixmapRef) {
        let offset = (
            (100 - icon_map.width() as i32) / 2,
            (100 - icon_map.height() as i32) / 2,
        );

        map.draw_pixmap(
            100 + 100 * position.0 + offset.0,
            100 + 100 * position.1 + offset.1,
            icon_map,
            &PixmapPaint {
                opacity: 255.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Bilinear,
            },
            Transform::identity(),
            None,
        );
    }

    fn add_border_number(position: (i32, i32), number: u32, map: &mut Pixmap) {
        let text_map = text::text_to_pixmap(&number.to_string(), &*text::ROBOTO, 85.0, (0, 0, 0));

        map.draw_pixmap(
            position.0,
            position.1,
            text_map.as_ref(),
            &PixmapPaint {
                opacity: 255.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Bilinear,
            },
            Transform::identity(),
            None,
        )
        .unwrap();
    }

    fn add_border(map: &mut Pixmap, game: &Game) {
        for x in 0..=game.width {
            let rect = Rect::from_xywh((x * 100) as f32, 0.0, 100.0, 100.0).unwrap();

            let color = if x % 2 == 0 {
                *BORDER_COLOR_DARK
            } else {
                *BORDER_COLOR_LIGHT
            };

            let paint = create_default_paint(color);

            map.fill_rect(rect, &paint, Transform::identity(), None);

            if x > 0 {
                let x_offset = if x > 9 { -4 } else { 18 };
                Self::add_border_number((x as i32 * 100 + x_offset, 7), x, map);
            }
        }
        for y in 1..=game.height {
            let rect = Rect::from_xywh(0.0, (y * 100) as f32, 100.0, 100.0).unwrap();

            let color = if y % 2 == 0 {
                *BORDER_COLOR_DARK
            } else {
                *BORDER_COLOR_LIGHT
            };

            let paint = create_default_paint(color);

            map.fill_rect(rect, &paint, Transform::identity(), None);

            let x_offset = if y > 9 { -4 } else { 18 };
            Self::add_border_number((x_offset, y as i32 * 100 + 7), y, map);
        }
    }

    fn outline_tiles(map: &mut Pixmap, game: &Game) {
        let paint = create_default_paint(*GRASS_OUTLINE_COLOR);

        for (y, x_row) in game.tiles.iter().enumerate() {
            for (x, tile) in x_row.iter().enumerate() {
                if tile.is_revealed {
                    continue;
                }
                if x > 0 && game.tiles[y][x - 1].is_revealed {
                    Self::draw_line(
                        map,
                        (100.0 + x as f32 * 100.0, 100.0 * y as f32 + 100.0),
                        100.0,
                        &GRASS_OUTLINE_COLOR,
                        LineType::Vertical,
                    );
                }
                if (x as i32) < game.width as i32 - 1 && game.tiles[y][x + 1].is_revealed {
                    Self::draw_line(
                        map,
                        (200.0 + x as f32 * 100.0, 100.0 * y as f32 + 100.0),
                        100.0,
                        &GRASS_OUTLINE_COLOR,
                        LineType::Vertical,
                    );
                }

                if y > 0 && game.tiles[y - 1][x].is_revealed {
                    Self::draw_line(
                        map,
                        (100.0 + x as f32 * 100.0, 100.0 * y as f32 + 100.0),
                        100.0,
                        &GRASS_OUTLINE_COLOR,
                        LineType::Horizontal,
                    );
                }

                if (y as i32) < game.height as i32 - 1 && game.tiles[y + 1][x].is_revealed {
                    Self::draw_line(
                        map,
                        (100.0 + x as f32 * 100.0, 100.0 * y as f32 + 200.0),
                        100.0,
                        &GRASS_OUTLINE_COLOR,
                        LineType::Horizontal,
                    );
                }

                if (y as i32) < game.height as i32 - 1 {
                    if (x as i32) < game.width as i32 - 1 && game.tiles[y + 1][x + 1].is_revealed {
                        let rect = Rect::from_xywh(
                            100.0 + (x + 1) as f32 * 100.0 - LINE_WIDTH_HALF,
                            100.0 * (y + 1) as f32 + 100.0 - LINE_WIDTH_HALF,
                            LINE_WIDTH,
                            LINE_WIDTH,
                        )
                        .unwrap();
                        map.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                    if x > 0 && game.tiles[y + 1][x - 1].is_revealed {
                        let rect = Rect::from_xywh(
                            100.0 + x as f32 * 100.0 - LINE_WIDTH_HALF,
                            100.0 * (y + 1) as f32 + 100.0 - LINE_WIDTH_HALF,
                            LINE_WIDTH,
                            LINE_WIDTH,
                        )
                        .unwrap();
                        map.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                }

                if y > 0 {
                    if (x as i32) < game.width as i32 - 1 && game.tiles[y - 1][x + 1].is_revealed {
                        let rect = Rect::from_xywh(
                            100.0 + (x + 1) as f32 * 100.0 - LINE_WIDTH_HALF,
                            100.0 * y as f32 + 100.0 - LINE_WIDTH_HALF,
                            LINE_WIDTH,
                            LINE_WIDTH,
                        )
                        .unwrap();
                        map.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                    if x > 0 && game.tiles[y - 1][x - 1].is_revealed {
                        let rect = Rect::from_xywh(
                            100.0 + x as f32 * 100.0 - LINE_WIDTH_HALF,
                            100.0 * y as f32 + 100.0 - LINE_WIDTH_HALF,
                            LINE_WIDTH,
                            LINE_WIDTH,
                        )
                        .unwrap();
                        map.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                }
            }
        }
    }

    fn add_mine_count(position: (i32, i32), number: u32, map: &mut Pixmap) {
        let text_map = text::text_to_pixmap(
            &number.to_string(),
            &*text::EB_GARAMOND,
            110.0,
            NUMBER_COLORS[number as usize],
        );

        map.draw_pixmap(
            118 + position.0 * 100,
            93 + position.1 * 100,
            text_map.as_ref(),
            &PixmapPaint {
                opacity: 255.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Bilinear,
            },
            Transform::identity(),
            None,
        )
        .unwrap();
    }

    fn add_top_bar(game_map: PixmapRef, game: &Game) -> Pixmap {
        let y_offset = (game_map.height() as f32 * 0.2) as u32;

        let mut map = Pixmap::new(game_map.width(), game_map.height() + y_offset).unwrap();

        map.draw_pixmap(
            0,
            y_offset as i32,
            game_map,
            &PixmapPaint {
                opacity: 255.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Bilinear,
            },
            Transform::identity(),
            None,
        );

        let rect = Rect::from_xywh(0.0, 0.0, map.width() as f32, y_offset as f32).unwrap();
        map.fill_rect(
            rect,
            &create_default_paint(*TOP_BAR_COLOR),
            Transform::identity(),
            None,
        );

        Self::draw_icon_scaled(
            (
                (80.0 * (game.width as f32 / 8.0)) as i32,
                (y_offset / 5) as i32,
            ),
            &mut map,
            FLAG_PIXMAP.as_ref(),
            (
                1.5 * (game.height as f32 / 8.0),
                1.5 * (game.height as f32 / 8.0),
            ),
        );

        let flag_count = text::text_to_pixmap(
            &(game.number_of_mines - game.placed_flag_count).to_string(),
            &*text::ROBOTO,
            80.0,
            (255, 255, 255),
        );

        Self::draw_icon_scaled(
            (
                (180.0 * (game.width as f32 / 8.0)) as i32,
                (y_offset / 5) as i32,
            ),
            &mut map,
            flag_count.as_ref(),
            (
                1.5 * (game.height as f32 / 8.0),
                1.5 * (game.height as f32 / 8.0),
            ),
        );

        Self::draw_icon_scaled(
            (
                (460.0 * (game.width as f32 / 8.0)) as i32,
                (y_offset / 5) as i32,
            ),
            &mut map,
            CLOCK_PIXMAP.as_ref(),
            (
                1.5 * (game.height as f32 / 8.0),
                1.5 * (game.height as f32 / 8.0),
            ),
        );

        let difference = game.last_move_time - game.time_started;

        let difference_text = format!(
            "{:02}:{:02}",
            difference.num_minutes(),
            difference.num_seconds() - difference.num_minutes() * 60
        );

        let difference_pixmap =
            text::text_to_pixmap(&difference_text, &*text::ROBOTO, 80.0, (255, 255, 255));

        Self::draw_icon_scaled(
            (
                (560.0 * (game.width as f32 / 8.0)) as i32,
                (y_offset / 5) as i32,
            ),
            &mut map,
            difference_pixmap.as_ref(),
            (
                1.5 * (game.height as f32 / 8.0),
                1.5 * (game.height as f32 / 8.0),
            ),
        );

        Self::draw_line(
            &mut map,
            (0.0, y_offset as f32),
            (100 + game.width * 100) as f32,
            &Color::BLACK,
            LineType::Horizontal,
        );
        map
    }

    fn draw_icon_scaled(
        position: (i32, i32),
        map: &mut Pixmap,
        icon_map: PixmapRef,
        scale: (f32, f32),
    ) {
        let scaled_x = (position.0 as f32 / scale.0) as i32;
        let scaled_y = (position.1 as f32 / scale.1) as i32;
        map.draw_pixmap(
            scaled_x,
            scaled_y,
            icon_map,
            &PixmapPaint {
                opacity: 255.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Bilinear,
            },
            Transform::from_scale(scale.0, scale.1),
            None,
        );
    }

    fn add_flowers(map: &mut Pixmap, game: &Game) {
        for (y, x_row) in game.tiles.iter().enumerate() {
            for (x, tile) in x_row.iter().enumerate() {
                if tile.is_revealed {
                    continue;
                }

                let flower_count: i32 = rand::thread_rng().gen_range(1..=3);
                for _ in 0..flower_count {
                    let flower_number = rand::thread_rng().gen_range(0..FLOWER_PIXMAPS.len());
                    let rotation: f32 = rand::thread_rng().gen_range(0.0..360.0);

                    let position = ((x + 1) as i32 * 100 + 50, (y + 1) as i32 * 100 + 50);

                    let scale: f32 = rand::thread_rng().gen_range(1.0..2.5);

                    let scaled_flower =
                        Self::scale_pixmap(FLOWER_PIXMAPS[flower_number].as_ref(), (scale, scale));

                    map.draw_pixmap(
                        position.0,
                        position.1,
                        scaled_flower.as_ref(),
                        &PixmapPaint {
                            opacity: 255.0,
                            blend_mode: BlendMode::SourceOver,
                            quality: FilterQuality::Bilinear,
                        },
                        Transform::from_rotate_at(rotation, position.0 as f32, position.1 as f32),
                        None,
                    );
                }
            }
        }
    }

    fn scale_pixmap(old_pixmap: PixmapRef, scale: (f32, f32)) -> Pixmap {
        let mut map = Pixmap::new(
            (old_pixmap.width() as f32 * scale.0) as u32,
            (old_pixmap.height() as f32 * scale.1) as u32,
        )
        .unwrap();

        map.draw_pixmap(
            0,
            0,
            old_pixmap,
            &PixmapPaint {
                opacity: 255.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Bilinear,
            },
            Transform::from_scale(scale.0, scale.1),
            None,
        );
        map
    }
}

impl MinesweeperDrawer for DefaultMinesweeperDrawer {
    fn draw_board(game: &Game) -> Pixmap {
        let mut map = Pixmap::new((game.width + 1) * 100, (game.height + 1) * 100).unwrap();

        for (y, x_row) in game.tiles.iter().enumerate() {
            for (x, tile) in x_row.iter().enumerate() {
                let rect =
                    Rect::from_xywh(((x + 1) * 100) as f32, ((y + 1) * 100) as f32, 100.0, 100.0)
                        .unwrap();

                if tile.is_revealed {
                    if game.state == GameState::Won {
                        let color = if (y + x) % 2 == 0 {
                            *WATER_COLOR_DARK
                        } else {
                            *WATER_COLOR_LIGHT
                        };
                        let paint = create_default_paint(color);
                        map.fill_rect(rect, &paint, Transform::identity(), None);
                    } else {
                        let color = if (y + x) % 2 == 0 {
                            *GROUND_COLOR_DARK
                        } else {
                            *GROUND_COLOR_LIGHT
                        };

                        let paint = create_default_paint(color);
                        map.fill_rect(rect, &paint, Transform::identity(), None);

                        if tile.adjacent_mines > 0 {
                            Self::add_mine_count(
                                (x as i32, y as i32),
                                tile.adjacent_mines,
                                &mut map,
                            )
                        }

                        if tile.is_mine {
                            Self::draw_icon(
                                (x as i32, y as i32),
                                &mut map,
                                EXPLOSION_PIXMAP.as_ref(),
                            );
                        }
                    }
                } else {
                    let color = if (y + x) % 2 == 0 {
                        *GRASS_COLOR_DARK
                    } else {
                        *GRASS_COLOR_LIGHT
                    };

                    let paint = create_default_paint(color);
                    map.fill_rect(rect, &paint, Transform::identity(), None);

                    if tile.is_flagged {
                        if game.state == GameState::Playing {
                            Self::draw_icon((x as i32, y as i32), &mut map, FLAG_PIXMAP.as_ref());
                        }
                        if game.state == GameState::Lost {
                            if tile.is_mine {
                                Self::draw_icon(
                                    (x as i32, y as i32),
                                    &mut map,
                                    FLAG_PIXMAP.as_ref(),
                                );
                            } else {
                                Self::draw_icon(
                                    (x as i32, y as i32),
                                    &mut map,
                                    X_MARK_PIXMAP.as_ref(),
                                );
                            }
                        }
                    }

                    if game.state == GameState::Lost && tile.is_mine {
                        Self::draw_icon((x as i32, y as i32), &mut map, WARNING_PIXMAP.as_ref());
                    }
                }
            }
        }

        Self::outline_tiles(&mut map, &game);
        if game.state == GameState::Won {
            Self::add_flowers(&mut map, &game);
        }
        Self::add_border(&mut map, &game);
        Self::add_border_line(&mut map, &game);

        if game.state == GameState::Lost {
            let rect = Rect::from_xywh(0.0, 0.0, map.width() as f32, map.height() as f32).unwrap();
            map.fill_rect(
                rect,
                &Paint {
                    shader: Shader::SolidColor(Color::from_rgba8(255, 0, 0, 100)),
                    blend_mode: BlendMode::SourceOver,
                    anti_alias: false,
                    force_hq_pipeline: true,
                },
                Transform::identity(),
                None,
            );
        }

        Self::add_top_bar(map.as_ref(), &game)
    }
}

fn create_default_paint<'a>(color: Color) -> Paint<'a> {
    Paint {
        shader: Shader::SolidColor(color),
        blend_mode: BlendMode::Source,
        anti_alias: false,
        force_hq_pipeline: true,
    }
}
