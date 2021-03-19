//! # Bevy MapEngine
//!
//! These docs to be filled in later. Meanwhile, see the README for
//! all of the top-level details.
//!
//! I've attempted to accurately comment on everything here inline,
//! and somewhat verbosely in the hopes that it might help someone
//! else new to Bevy.
//!
//!
//!

// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// These are used for creating the map texture
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

pub use map_space::{MapSpace, MapSpaceRefreshNeeded};

/*----------------------------------------------------------------------------*/

/// An internal collection of systems which handles loading tiles from
/// the given assets folder, and sets the Stage State to Running when
/// the tiles are all ready.
mod tileloader_systems;

/// Our collection of systems for collecting MapSpace entities from the
/// World and drawing them on our map Sprite to be rendered by Bevy.
mod map_systems;

/// Structs which define the MapSpace component type
mod map_space;

/*----------------------------------------------------------------------------*/

/// Bevy groups systems into stages. Our mapengine
/// runs in its own stage, and this is its name.
/// See main() for how this is actually used.
const MAPENGINE_STAGE: &str = "mapengine_stage";

/// Bevy does "lazy" loading of assets. We switch from the
/// Loading state to Running state when all of the tile images
/// are actually loaded.
#[derive(Clone)]
pub enum MapEngineState {
    Loading,
    Verifying,
    Running,
}

/// This is for the global resource that holds our map information.
pub struct MapEngineMap {
    /// The actual texture to be drawn on
    texture: Texture,
    /// Width of map in spaces (texture width = cols × space_width_pixels)
    cols: i32,
    /// Height of map in spaces (texture height = rows × space_height_pixels)
    rows: i32,
    /// Each space must be the same; keeping it here saves us reading it later.
    space_width_pixels: usize,
    /// Each space must be the same; keeping it here saves us reading it later.
    space_height_pixels: usize,
}

impl Default for MapEngineMap {
    /// default to an empty texture
    fn default() -> Self {
        MapEngineMap {
            // We start with the minimum possible texture size: 1×1
            // FUTURE have a reasonable default and make configurable
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

/// This component tags a sprite as map sprite
pub struct MapEngineSprite;

/*----------------------------------------------------------------------------*/

pub struct MapEnginePlugin;

impl Plugin for MapEnginePlugin {
    fn build(&self, app: &mut AppBuilder) {
        // A stash of handles to our image tiles, so we can use them everywhere.
        app.init_resource::<tileloader_systems::MapEngineTileHandles>()
            // This adds a "Stage" (basically, a group of systems) set up to handle our
            // various "States". Our stage, used in the MapEngine, will run right after
            // the default UPDATE stage. This is important because otherwise we will miss
            // changes to MapSpace entities done in the plugin user's code.
            // See https://bevy-cheatbook.github.io/basics/stages.html for more on stages.
            .add_stage_after(
                stage::UPDATE,
                MAPENGINE_STAGE,
                StateStage::<MapEngineState>::default(),
            )
            // This global resource tracks the state used in this stage.
            // We set it to Loading to start, of course.
            .add_resource(State::new(MapEngineState::Loading))
            // And this global resource holds the texture for our map.
            .add_resource(MapEngineMap::default())
            // This stage happens once when entering the Loading state (that is, right away)
            .on_state_enter(
                MAPENGINE_STAGE,
                MapEngineState::Loading,
                tileloader_systems::load_tiles_system.system(),
            )
            // And this stage runs every frame while still in Loading state
            // (and is responsible for changing the state to Checking when ready)
            .on_state_update(
                MAPENGINE_STAGE,
                MapEngineState::Loading,
                tileloader_systems::wait_for_tile_load_system.system(),
            )
            // This stage makes sure that our tiles are valid and stores information
            // about them in the global MapEngineMap resource, and then advances
            // the state to Running. It exits on failure; we could get even more
            // fancy and instead have an Error state which presents error messages in-game.
            .on_state_enter(
                MAPENGINE_STAGE,
                MapEngineState::Verifying,
                tileloader_systems::verify_tiles_system.system(),
            )
            // When we get to the Running state, add our map sprite
            .on_state_enter(
                MAPENGINE_STAGE,
                MapEngineState::Running,
                map_systems::create_map_sprite_system.system(),
            )
            // This system runs every frame once we are in the Running state.
            // Because it happens all the time, it needs to be careful to not
            // do slow things. See the code in the maptexture_update_system itself.
            .on_state_update(
                MAPENGINE_STAGE,
                MapEngineState::Running,
                map_systems::maptexture_update_system.system(),
            );
        // FUTURE add a validator which runs periodically and checks for overlapping MapSpaces?
        // NEXT add a system which takes mouse events and translates them into new events that
        // correspond to the mapspace location (enter, exit, click -- maybe motion?)
        // NEXT possibly also a global resource for current hovered or selected mapspace entity?
        // FUTURE map scrolling (with the mouse stuff still working!)
        // FUTURE map zooming (with the mouse stuff still working!)
    }
}
