/// In our current implementation, the visible map is handled as
/// one giant sprite. This module holds the struct which defines
/// a Sprite to be MapEngineSprite, and a global resource which
/// holds the configuration for all such sprites.
/*----------------------------------------------------------------------------*/
//

// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// These are used for creating the map texture
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

/*----------------------------------------------------------------------------*/

/// This component tags a sprite as map sprite
pub struct MapEngineSprite;

/*----------------------------------------------------------------------------*/

/// This is for the global resource that holds our map information.
pub struct Map {
    /// The actual texture to be drawn on
    pub texture: Texture,
    /// Width of map in spaces (texture width = cols × space_width_pixels)
    pub cols: i32,
    /// Height of map in spaces (texture height = rows × space_height_pixels)
    pub rows: i32,
    /// Each space must be the same; keeping it here saves us reading it later.
    pub space_width_pixels: usize,
    /// Each space must be the same; keeping it here saves us reading it later.
    pub space_height_pixels: usize,
}

impl Default for Map {
    /// default to an empty texture
    fn default() -> Self {
        Map {
            /// We start with the minimum possible texture size: 1×1
            /// FUTURE have a reasonable default and make configurable
            texture: Texture::new_fill(
                Extent3d::new(1, 1, 1),
                TextureDimension::D2,
                &[0, 0, 0, 0],
                TextureFormat::Rgba8UnormSrgb,
            ),
            cols: 0,
            rows: 0,
            space_width_pixels: 0,
            space_height_pixels: 0,
        }
    }
}
