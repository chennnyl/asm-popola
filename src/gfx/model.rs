use sdl2::pixels::Color;
use crate::inter::mmio;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub colors: [Color; mmio::PALETTE_LENGTH]
}
#[derive(Debug, Clone, Copy)]

pub struct Tile {
    pub pixels: [u8; mmio::TILE_SIZE]
}
#[derive(Debug, Copy, Clone)]

pub struct Tilemap {
    pub tiles: [Tile; mmio::TILEMAP_LENGTH*mmio::TILEMAP_LENGTH]
}
#[derive(Debug, Copy, Clone)]

pub struct Background {
    pub tiles: [u8; mmio::BG_SIZE]
}
#[derive(Debug, PartialEq, Copy, Clone)]

pub enum SpriteSize {
    X8,
    X16,
    X32,
    X64
}

impl From<u8> for SpriteSize {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::X8,
            1 => Self::X16,
            2 => Self::X32,
            _ => Self::X64
        }
    }
}

impl SpriteSize {
    pub fn size(sprite_size: SpriteSize) -> usize {
        match sprite_size {
            SpriteSize::X8 => 64,
            SpriteSize::X16 => 256,
            SpriteSize::X32 => 1024,
            SpriteSize::X64 => 4096
        }
    }
    pub fn pitch(sprite_size: SpriteSize) -> u32 {
        match sprite_size {
            SpriteSize::X8 => 8,
            SpriteSize::X16 => 16,
            SpriteSize::X32 => 32,
            SpriteSize::X64 => 64
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]

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

#[derive(Debug, PartialEq, Copy, Clone)]

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