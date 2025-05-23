use quicksilver::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
struct Tile {
    pos: Vector,
    glyph: char,
    color: Color,
}

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector,
    glyph: char,
    color: Color,
    hp: i32,
    max_hp: i32,
}

struct Game {
    title: Asset<Image>,
    mononoki_font_info: Asset<Image>,
    square_font_info: Asset<Image>,
    map_size: Vector,
    map: Vec<Tile>,
    entities: Vec<Entity>,
    player_id: usize,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
    confirming_exit: bool,
    confirm_exit_font: Asset<Font>,
}

impl State for Game {
    fn new() -> Result<Self> {
        let font_mononoki = "mononoki-Regular.ttf";
        let font_square = "square.ttf";
        
        let confirm_exit_font = Asset::new(Font::load(font_mononoki));

        let map_size = Vector::new(20, 15);
        let map = generate_map(map_size);
        let mut entities = generate_entities();
        let player_id = entities.len();
        entities.push(Entity {
            pos: Vector::new(5,3),
            glyph: '@',
            color: Color::BLUE,
            hp: 3,
            max_hp: 5,
        });
        
        let game_glyphs = "#@g.%";
        let tile_size_px = Vector::new(24,24);
        
        let tileset = Asset::new(Font::load(font_square).and_then(move |font| {
            let tiles = font.render(game_glyphs, &FontStyle::new(tile_size_px.y, Color::WHITE))?;
            let mut tileset = HashMap::new();
            for (index, glyph) in game_glyphs.chars().enumerate() {
                let pos = (index as i32 * tile_size_px.x as i32, 0);
                let tile = tiles.subimage(Rectangle::new(pos, tile_size_px));
                tileset.insert(glyph, tile);
            }
            Ok(tileset)
        }));

        let title = Asset::new(Font::load(font_mononoki).and_then(|font| {
            font.render("Rogue Like", &FontStyle::new(72.0, Color::BLACK))
        }));

        let mononoki_font_info = Asset::new(Font::load(font_mononoki).and_then(|font| {
            font.render(
                "Mononoki font by Matthias Tellen, terms: SIL Open Font License 1.1",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let square_font_info = Asset::new(Font::load(font_square).and_then(move |font| {
            font.render(
                "Square font by Wouter Van Oortmerssen, terms: CC BY 3.0",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        Ok(Self {
            title,
            mononoki_font_info,
            square_font_info,
            map_size,
            map,
            entities,
            player_id,
            tileset,
            tile_size_px,
            confirming_exit: false,
            confirm_exit_font,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        // Handle exit confirmation
        use ButtonState::*;

        if self.confirming_exit {
            if window.keyboard()[Key::Y] == Pressed {
                window.close();
            } else if window.keyboard()[Key::N] == Pressed || window.keyboard()[Key::Escape] == Pressed {
                self.confirming_exit = false;
            }
        } 
        // Handle normal game controls
        else {
            let player = &mut self.entities[self.player_id];
            
            // Movement controls (using was_pressed for single moves)
            if window.keyboard()[Key::Left] == Pressed {
                player.pos.x = (player.pos.x - 1.0).max(0.0);
            }
            if window.keyboard()[Key::Right] == Pressed {
                player.pos.x = (player.pos.x + 1.0).min(self.map_size.x - 1.0);
            }
            if window.keyboard()[Key::Up] == Pressed {
                player.pos.y = (player.pos.y - 1.0).max(0.0);
            }
            if window.keyboard()[Key::Down] == Pressed {
                player.pos.y = (player.pos.y + 1.0).min(self.map_size.y - 1.0);
            }
            
            // Open exit confirmation
            if window.keyboard()[Key::Escape] == Pressed {
                self.confirming_exit = true;
            }
        }
        Ok(())
    }
    // ... keep your existing draw() implementation exactly the same ...
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;
        self.title.execute(|image| {
            window.draw(
                &image
                    .area()
                    .with_center((window.screen_size().x as i32 / 2, 40)),
                Img(&image),
            );
            Ok(())
        })?;

        self.mononoki_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window.screen_size().y as i32 - 60)),
                Img(&image),
            );
            Ok(())
        })?;

        self.square_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window.screen_size().y as i32 - 30)),
                Img(&image),
            );
            Ok(())
        })?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new(175, 120);

        let (tileset, map) = (&mut self.tileset, &self.map);
        tileset.execute(|tileset| {
            for tile in map.iter() {
                if let Some(image) = tileset.get(&tile.glyph) {
                    let pos_px = tile.pos.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(pos_px + offset_px, image.area().size()),
                        Blended(&image, tile.color),
                    );
                }
            }
            Ok(())
        })?;

        let (tileset, entities) = (&mut self.tileset, &self.entities);
        tileset.execute(|tileset| {
            for entity in entities.iter() {
                if let Some(image) = tileset.get(&entity.glyph) {
                    let pos_px = offset_px + entity.pos.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, entity.color)
                    )
                }
            }
            Ok(())
        })?;

        let player = &self.entities[self.player_id];
        let full_health_width_px = 100.0;
        let current_health_width_px =
            (player.hp as f32 / player.max_hp as f32) * full_health_width_px;
        let map_size_px = self.map_size.times(tile_size_px);
        let health_bar_pos_px = offset_px + Vector::new(map_size_px.x, 0.0);

        window.draw(
            &Rectangle::new(health_bar_pos_px, (full_health_width_px, tile_size_px.y)),
            Col(Color::RED.with_alpha(0.5)),
        );
        window.draw(
            &Rectangle::new(health_bar_pos_px, (current_health_width_px, tile_size_px.y)),
            Col(Color::RED),
        );

        // Add confirmation dialog drawing
        if self.confirming_exit {
            self.confirm_exit_font.execute(|font| {
                let text = font.render(
                    "Are you sure you want to quit? (Y/N)",
                    &FontStyle::new(32.0, Color::BLACK),
                )?;
                let pos = Vector::new(100.0, 100.0);
                window.draw(&text.area().translate(pos), Img(&text));
                Ok(())
            })?;
        }

        Ok(())
    }
}

// ... keep your existing generate_map(), generate_entities(), and main() functions ...
fn generate_map(size:Vector) -> Vec<Tile> {
    let width = size.x as usize;
    let height = size.y as usize;
    let mut map = Vec::with_capacity(width * height);
    for x in 0..width {
        for y in 0..height {
            let mut tile = Tile {
                pos: Vector::new(x as f32, y as f32),
                glyph: '.',
                color: Color::BLACK,
            };

            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                tile.glyph = '#';
            };
            map.push(tile);
        }
    }
    map
}

fn generate_entities() -> Vec<Entity> {
    vec![
        Entity {
            pos: Vector::new(9,6),
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
        },
        Entity {
            pos: Vector::new(9,6),
            glyph: 'g',
            color: Color::RED,
            hp:1,
            max_hp: 1,
        },
        Entity {
            pos: Vector::new(2,4),
            glyph: 'g',
            color: Color::RED,
            hp:1,
            max_hp: 1,
        },
        Entity {
            pos: Vector::new(7,5),
            glyph: '%',
            color: Color::PURPLE,
            hp:0,
            max_hp: 0,
        },
        Entity {
            pos: Vector::new(4,8),
            glyph: '%',
            color: Color::PURPLE,
            hp:0,
            max_hp: 0,
        },
    ]
}
fn main() {
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");
    let settings = Settings {
        scale: quicksilver::graphics::ImageScaleStrategy::Blur,
        ..Default::default()
    };
    run::<Game>("Rogue Like", Vector::new(800,600), settings);
}

