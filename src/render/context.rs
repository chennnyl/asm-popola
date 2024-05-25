use sdl2::render::{TextureCreator, WindowCanvas, Texture};
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use crate::gfx::*;
use crate::inter::{mmio::*, gfx::*};

pub(crate) struct RenderContext {
    canvas: Rc<RefCell<WindowCanvas>>,
    texture_creator: RefCell<TextureCreator<WindowContext>>,
    vrammodel: VRAMModel
}

impl RenderContext {
    fn render_to_sprite(
        &self,
        sprite_texture: Rc<RefCell<Texture<'_>>>,
        sprite_index: usize,
    ) {
        let mut sprite_texture = sprite_texture.borrow_mut();

        let sprite = &self.vrammodel.sprites[sprite_index];
        let properties = sprite.properties;
        let tilemap = &self.vrammodel.tilemaps[properties.tilemap_index as usize];
        let palette = &self.vrammodel.palettes[properties.palette_index as usize];

        let pitch = SpriteSize::pitch(properties.size);
        let tile_pitch = pitch as usize / TILE_LENGTH;

        let tile_count = (tile_pitch * tile_pitch) as u8;

        let tiles = &tilemap.tiles[sprite.gfx_start as usize..(sprite.gfx_start + tile_count) as usize];

        let screen_destination = Rect::new(
            sprite.location.0 as i32, sprite.location.1 as i32, pitch, pitch
        );

        println!("{:?}", screen_destination);

        tiles.iter()
            .enumerate()
            .for_each(|(index, tile)| {
                // relative location of tile within sprite
                let (tx, ty) = (index % tile_pitch, index / tile_pitch);
                let destination_rect = Rect::new(
                    (tx*tile_pitch) as i32, (ty*tile_pitch) as i32,
                    tile_pitch as u32, tile_pitch as u32
                );
                // convert the tile into an array of bytes representing the pixel data
                let absolute_colors = tile.pixels
                    .iter()
                    .map(|palette_index| {
                        let color = palette.colors[*palette_index as usize];
                        [color.r, color.g, color.b]
                    })
                    .flatten()
                    .collect::<Vec<u8>>();

                println!("{:?}", absolute_colors);
                // blit onto the texture
                sprite_texture.update(
                    destination_rect, &absolute_colors, 3*TILE_LENGTH
                ).unwrap();
            });

        self.canvas
            .borrow_mut()
            .copy(&sprite_texture, None, screen_destination)
            .unwrap();
    }

}

#[cfg(test)]
mod tests {
    use sdl2::event::Event;
    use super::*;

    fn dummy_tile() -> Tile {
        Tile { pixels: [0; TILE_SIZE] }
    }

    fn dummy_palette() -> Palette {
        Palette { colors: [Color::BLACK; PALETTE_LENGTH] }
    }
    fn dummy_tilemap() -> Tilemap {
        Tilemap { tiles: [dummy_tile(); TILEMAP_LENGTH*TILEMAP_LENGTH] }
    }
    fn dummy_background() -> Background {
        Background { tiles: [0; BG_SIZE] }
    }

    fn dummy_sprite() -> Sprite {
        Sprite {
            properties: SpriteProperties {
                tilemap_index: 0, palette_index: 0, size: SpriteSize::X8, priority: 0
            },
            location: (0, 0),
            gfx_start: 0,
            info: 0
        }
    }

    #[test]
    fn test_render_sprite() {
        let mut palettes = [dummy_palette(); PALETTE_COUNT];
        palettes[0] = Palette { colors: [
            Color::BLACK,
            Color::RED,
            Color::GREEN,
            Color::BLUE,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
            Color::RED,
        ] };
        let mut tilemaps = [dummy_tilemap(); TILEMAP_COUNT];
        let mut tiles = [dummy_tile(); TILEMAP_LENGTH*TILEMAP_LENGTH];
        tiles[0] = Tile {
            pixels: [
                1, 1, 1, 1, 1, 1, 1, 1, // !!!!!!!!
                0, 1, 2, 2, 2, 2, 1, 0, // _!@@@@!_
                0, 0, 1, 3, 3, 1, 0, 0, // __!##!__
                0, 0, 0, 2, 2, 0, 0, 0, // ___@@___
                0, 0, 0, 2, 2, 0, 0, 0, // ___@@___
                0, 0, 1, 3, 3, 1, 0, 0, // __!##!__
                0, 1, 2, 2, 2, 2, 1, 0, // _!@@@@!_
                1, 1, 1, 1, 1, 1, 1, 1, // !!!!!!!!
            ]
        };
        tilemaps[0].tiles = tiles;

        let backgrounds = [dummy_background(); BG_COUNT];
        let mut sprites = [dummy_sprite(); SPRITE_COUNT];

        sprites[0] = Sprite {
            properties: SpriteProperties {
                tilemap_index: 0,
                size: SpriteSize::X8,
                palette_index: 0,
                priority: 0
            },
            location: (128, 128),
            gfx_start: 0,
            info: 0
        };

        let fake_vram = VRAMModel {
            palettes, tilemaps, backgrounds, sprites
        };

    }
}