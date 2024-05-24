use sdl2::pixels::Color;
use crate::inter::mmio;

#[derive(Debug)]
pub struct Palette {
    pub colors: [Color; mmio::PALETTE_LENGTH]
}
#[derive(Debug, Clone)]

pub struct Tile {
    pub pixels: [u8; mmio::TILE_SIZE]
}
#[derive(Debug)]

pub struct Tilemap {
    pub tiles: [Tile; mmio::TILEMAP_LENGTH*mmio::TILEMAP_LENGTH]
}
#[derive(Debug)]

pub struct Background {
    pub tiles: [u8; mmio::BG_SIZE]
}
#[derive(Debug, PartialEq, Copy, Clone)]

pub enum SpriteSize {
    x8, x16, x32, x64
}

impl From<u8> for SpriteSize {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::x8,
            1 => Self::x16,
            2 => Self::x32,
            _ => Self::x64
        }
    }
}

impl SpriteSize {
    pub fn size(sprite_size: SpriteSize) -> usize {
        match sprite_size {
            SpriteSize::x8 => 64,
            SpriteSize::x16 => 256,
            SpriteSize::x32 => 1024,
            SpriteSize::x64 => 4096
        }
    }
    pub fn pitch(sprite_size: SpriteSize) -> u32 {
        match sprite_size {
            SpriteSize::x8 => 8,
            SpriteSize::x16 => 16,
            SpriteSize::x32 => 32,
            SpriteSize::x64 => 64
        }
    }
}

#[derive(Debug, PartialEq)]

pub struct SpriteProperties {
    pub tilemap_index: u8,
    pub size: SpriteSize,
    pub palette_index: u8,
    pub priority: u8
}

impl From<u8> for SpriteProperties {
    fn from(value: u8) -> Self {
        Self {
            tilemap_index: value >> 7,
            size: SpriteSize::from((value >> 5) & 0b11),
            palette_index: (value >> 2) & 0b111,
            priority: value & 0b11
        }
    }
}

#[derive(Debug, PartialEq)]

pub struct Sprite {
    pub properties: SpriteProperties,
    pub location: (u8, u8),
    pub gfx_start: u8,
    pub info: u8
}

#[derive(Debug)]
pub struct VRAMModel {
    pub palettes: [Palette; mmio::PALETTE_COUNT],
    pub tilemaps: [Tilemap; mmio::TILEMAP_COUNT],
    pub backgrounds: [Background; mmio::BG_COUNT],
    pub sprites: [Sprite; mmio::SPRITE_COUNT]
}