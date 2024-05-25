use crate::gfx::*;
use crate::inter::mmio::*;
use devola::stdlib;
use devola::vm::Devola;
use devola::utility::*;

impl VRAMModel {
    fn empty_tile() -> Tile {
        Tile { pixels: [0; TILE_SIZE] }
    }

    fn empty_palette() -> Palette {
        Palette { colors: [Color { r: 0, g: 0, b: 0 }; PALETTE_LENGTH] }
    }
    fn empty_tilemap() -> Tilemap {
        Tilemap { tiles: [VRAMModel::empty_tile(); TILEMAP_LENGTH*TILEMAP_LENGTH] }
    }
    fn empty_background() -> Background {
        Background { tiles: [0; BG_SIZE] }
    }

    fn empty_sprite() -> Sprite {
        Sprite {
            properties: SpriteProperties {
                tilemap_index: 0, palette_index: 0, size: SpriteSize::X8, priority: 0
            },
            location: (0, 0),
            gfx_start: 0,
            info: 0
        }
    }

    pub fn empty_vram() -> VRAMModel {
        let palettes = [VRAMModel::empty_palette(); PALETTE_COUNT];
        let tilemaps = [VRAMModel::empty_tilemap(); TILEMAP_COUNT];
        // let tiles = [VRAMModel::empty_tile(); TILEMAP_LENGTH*TILEMAP_LENGTH];

        let backgrounds = [VRAMModel::empty_background(); BG_COUNT];
        let sprites = [VRAMModel::empty_sprite(); SPRITE_COUNT];

        VRAMModel {
            palettes, tilemaps, backgrounds, sprites
        }
    }

    pub fn enable_sprite(&mut self, sprite_index: u8) {
        let render_info = self.sprites[sprite_index as usize].info;

        self.sprites[sprite_index as usize].info = render_info | 0b00000001;
    }
    pub fn disable_sprite(&mut self, sprite_index: u8) {
        let render_info = self.sprites[sprite_index as usize].info;

        self.sprites[sprite_index as usize].info = render_info & 0b11111110;
    }
}


pub trait VRAMDeserialize: Sized {

    /// Returns a tuple describing the start of the region in VRAM and the size of a member
    fn dimensions() -> (u16, u16);

    /// Takes a slice of byte data and interprets it
    fn deserialize(data: &[u8]) -> Self;

    /// Get the nth member in VRAM
    fn get_nth(devola: &mut Devola, index: u16) -> Self {
        let (start, size) = Self::dimensions();
        let data = stdlib::memgetn(devola, start+size*index, size);
        Self::deserialize(data)
    }

}

pub fn rgb15_to_color(color_word: u16) -> Color {
    Color {
        r: 8 * (color_word >> 10) as u8,
        g: 8 * ((color_word >> 5) & 0x1F) as u8,
        b: 8 * (color_word & 0x1F) as u8,
    }
}
pub fn color_to_rgb15(color: Color) -> u16 {
    (((color.r as u16) / 8) << 10) | (((color.g as u16) / 8) << 5) | ((color.b as u16) / 8)
}

// Palette
impl VRAMDeserialize for Palette {
    fn dimensions() -> (u16, u16) {
        (PALETTE_START, PALETTE_SIZE as u16)
    }
    fn deserialize(data: &[u8]) -> Palette {
        // RGB15 are laid out as
        // MSB      LSB
        // 0rrrrrgg gggbbbbb
        // =>  red     = word >> 10
        //     green   = (word >> 5) & 0x1F
        //     blue    = word & 0x1F
        let mut colors: [Color; PALETTE_LENGTH] = [Color { r: 0, g: 0, b: 0 }; PALETTE_LENGTH];
        (0..PALETTE_LENGTH).for_each(
            |i| {
                let index = i*2;
                let (hi, lo) = (data[index], data[index+1]);
                let color_word = build_u16(hi, lo);
                colors[i] = rgb15_to_color(color_word);
            }
        );

        Palette {
            colors
        }
    }
}
// Tile
impl VRAMDeserialize for Tile {

    /// `Tile` should never have `dimensions()` called
    fn dimensions() -> (u16, u16) {
        unreachable!();
    }

    fn deserialize(data: &[u8]) -> Tile {
        Tile {
            pixels: data.try_into().unwrap()
        }
    }
}

// Tilemap
impl VRAMDeserialize for Tilemap {
    fn dimensions() -> (u16, u16) {
        (TILEMAP_START, TILEMAP_SIZE as u16)
    }

    fn deserialize(data: &[u8]) -> Tilemap {

        Tilemap {
            tiles: data.chunks(TILE_SIZE)
                    .map(Tile::deserialize)
                    .collect::<Vec<Tile>>()
                    .try_into()
                    .unwrap()
        }
    }
}
// Background
impl VRAMDeserialize for Background {
    fn dimensions() -> (u16, u16) {
        (BG_START, BG_SIZE as u16)
    }
    
    fn deserialize(data: &[u8]) -> Background {
        Background {
            tiles: data.try_into().unwrap()
        }
    }
}

// Sprite
impl VRAMDeserialize for Sprite {
    fn dimensions() -> (u16, u16) {
        (SPRITE_START, SPRITE_SIZE as u16)
    }
    fn deserialize(data: &[u8]) -> Sprite {
        Sprite {
            properties: SpriteProperties::from(data[0]),
            location: (data[1], data[2]),
            gfx_start: data[3],
            info: data[4]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_deserialize() {
        let data: [u8; 5] = [
            0b0_10_001_01,
            128, 32,
            0,
            0
        ];
        assert_eq!(
            Sprite::deserialize(&data),
            Sprite {
                properties: SpriteProperties {
                    tilemap_index: 0,
                    size: SpriteSize::X32,
                    palette_index: 1,
                    priority: 1
                },
                location: (128, 32),
                gfx_start: 0,
                info: 0
            }
        )
    }
}