use devola::vm::*;

pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 224;

// MMIO+0x0, 0x1 reserved

// all sprites can be enabled, disabled by toggling the high bit (+/- 128)
// individual sprites can be enabled or disabled via the 7 lower bits
pub const SPRITE_TOGGLES: u16 = MMIO+0x2;

// [enable 0|tilemap 0|palette 2|palette 1|palette 0|bg 1|bg 0|unused]
pub const BG_SETTINGS: u16 = MMIO+0x3;
// VRAM mapping -- 48KiB
pub const VRAM: u16 = 0x6000;
// Palettes
pub const PALETTE_START: u16 = VRAM+0x0;
pub const COLOR_SIZE: usize = 2; // 15-bit color
pub const PALETTE_LENGTH: usize = 16;
pub const PALETTE_SIZE: usize = COLOR_SIZE*PALETTE_LENGTH;
pub const PALETTE_COUNT: usize = 8;
pub const PALETTE_OFFSET: u16 = PALETTE_START+(PALETTE_SIZE*PALETTE_COUNT) as u16;

// Tilemaps
pub const TILEMAP_START: u16 = PALETTE_OFFSET;
pub const TILE_LENGTH: usize = 8;
pub const TILE_SIZE: usize = TILE_LENGTH*TILE_LENGTH; // 8x8 tiles
pub const TILEMAP_LENGTH: usize = 16; // 16x16 tilemaps; 256 tiles per tilemap
pub const TILEMAP_SIZE: usize = TILE_SIZE*TILEMAP_LENGTH*TILEMAP_LENGTH;
pub const TILEMAP_COUNT: usize = 2;
pub const TILEMAP_OFFSET: u16 = TILEMAP_START+(TILEMAP_SIZE*TILEMAP_COUNT) as u16;

// Backgrounds
pub const BG_START: u16 = TILEMAP_OFFSET;
pub const BG_WIDTH: usize = SCREEN_WIDTH as usize/TILE_LENGTH;
pub const BG_HEIGHT: usize = SCREEN_HEIGHT as usize/TILE_LENGTH;
pub const BG_SIZE: usize = BG_WIDTH*BG_HEIGHT;
pub const BG_COUNT: usize = 4;
pub const BG_OFFSET: u16 = BG_START+(BG_SIZE*BG_COUNT) as u16;

// Sprites

// Sprites are laid out as follows:
// Properties: 1 byte [tilemap 0|size 1|size 0|palette 2|palette 1|palette 0|priority 1|priority 0]
//                     tilemap: 0 or 1, selects tilemap to be used
//                               size: 0 (8x8) 1 (16x16) 2 (32x32) 3 (64x64)
//                                             palette: 0-7, selects palette to use
//                                                                           priority: 0-3, higher priority is drawn over lower
// Location: 2 bytes (x then y)
// Start index: 1 byte
// Rendering info: 1 byte (currently unused)
pub const SPRITE_START: u16 = BG_OFFSET;
pub const SPRITE_SIZE: usize = 5;
pub const SPRITE_COUNT: usize = 128;
// pub const SPRITE_OFFSET: u16 = SPRITE_START+(SPRITE_SIZE*SPRITE_COUNT) as u16;