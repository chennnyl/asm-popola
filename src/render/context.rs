use crate::inter::mmio::*;
use crate::gfx::*;

use pixels::Pixels;

pub(crate) struct RenderContext {
    pub vrammodel: VRAMModel,
    pixels: Pixels,
}

impl RenderContext {
    pub fn new(pixels: Pixels) -> RenderContext {
        RenderContext { vrammodel: VRAMModel::empty_vram(), pixels }
    }

    pub fn render(&mut self) {
        let frame = self.pixels.frame_mut();
        for (pi, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x  = pi % SCREEN_WIDTH as usize;
            let val = (8*(x/8) % 256) as u8;
            let color = [val, 0x00, 0x00, 0xff];
            pixel.copy_from_slice(&color);
        }
        for sprite in &self.vrammodel.sprites {
            if sprite.enabled() {
                RenderContext::render_sprite(&self.vrammodel, sprite, frame);
            }
        }
        self.pixels.render().unwrap();
    }

    fn render_sprite(vram: &VRAMModel, sprite: &Sprite, frame: &mut [u8]) {
        let properties = sprite.properties;
        let tilemap = vram.tilemaps[properties.tilemap_index as usize];
        let palette = vram.palettes[properties.palette_index as usize];

        let pitch = SpriteSize::pitch(properties.size); // width of the whole sprite
        let tile_pitch = pitch as usize / TILE_LENGTH; // width of the sprite in tiles

        let tile_count = (tile_pitch * tile_pitch) as u8;

        let tiles = &tilemap.tiles[sprite.gfx_start as usize..(sprite.gfx_start + tile_count) as usize];

        let (top_x, top_y) = sprite.location;

        tiles.iter()
            .enumerate()
            .for_each(|(index, tile)| {
                // convert the tile into an array of bytes representing the pixel data
                let tile_flat: Vec<u8> = tile.pixels
                    .iter()
                    .map(|palette_index| {
                        let color = palette.colors[*palette_index as usize];
                        [color.r, color.g, color.b, 0xFF]
                    })
                    .flatten()
                    .collect();

                let (tx, ty) = (index % tile_pitch, index / tile_pitch);

                let (absolute_x, absolute_y) = (top_x as usize + TILE_LENGTH*tx, top_y as usize + TILE_LENGTH*ty);

                tile_flat.chunks_exact(TILE_LENGTH*4)
                    .enumerate()
                    .for_each(|(line_index, line)| {
                        let linear_start = SCREEN_WIDTH as usize*(absolute_y+line_index)*4 + absolute_x*4;
                        frame[linear_start..linear_start+TILE_LENGTH*4].copy_from_slice(line);
                    });
            });
    }

}

#[cfg(test)]
mod tests {
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