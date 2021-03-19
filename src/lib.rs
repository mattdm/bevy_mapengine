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

// Standard rust things...
use std::cmp;

/*----------------------------------------------------------------------------*/

/// An internal collection of systems which handles loading tiles from
/// the given assets folder, and sets the Stage State to Running when
/// the tiles are all ready.
mod tileloader_systems;

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

/// This is a Bevy Component that defines an Entity as representing
/// a space on our map, and holds the location and the tile image
/// to use. Note that these are meant to represent fixed locations
/// on the map; x and y should not change. Note also that I haven't
/// decided on how to do layering, so duplicating (col,row) will
/// lead to unpredictable results.
///
/// FUTURE consider making col and row read-only using the readonly crate
/// FUTURE make texture_handle a Vec, and draw in order?
/// The other layering approach (adding depth, allowing multiple col,row)
/// has the disadvantage that we need to find all of the entities to draw.
// FIXME texture handle should not need to be public, so we need a constructor
#[derive(Debug)]
pub struct MapSpace {
    /// Column (x) position of this tile on the map. 0 is on the left.
    pub col: i32,
    /// Row (y) position of this tile on the map. 0 is at the top.
    pub row: i32,
    /// load with, for example, `asset_server.get_handle("terrain/grass1.png")`
    pub texture_handle: Handle<Texture>,
}

/// This component signals that a MapSpace needs to be refreshed.
/// This is a hack until https://github.com/bevyengine/bevy/pull/1471 is implemented.
// TODO make not public?
pub struct MapSpaceRefreshNeeded;

/// Our list of handles to tile images is stored as a global
/// Bevy resource so we can use them in various systems. In Bevy,
/// these global resources are located by type, so we need a custom
/// type to do this.
#[derive(Default)]
pub struct MapEngineTileHandles {
    handles: Vec<HandleUntyped>,
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
struct MapEngineSprite;

/*----------------------------------------------------------------------------*/

/// Ripped from bevy_sprite/src/texture_atlas_builder.rs.
///
/// This doesn't really copy actual GPU textures. It copies bits
/// in a Vec representing RGBA data. This is not going way we want
/// to do this always, but we are waiting on
/// https://github.com/bevyengine/bevy/issues/1207#issuecomment-800602680
/// for a real solution.
fn copy_texture(
    target_texture: &mut Texture,
    source_texture: &Texture,
    rect_x: usize,
    rect_y: usize,
) {
    let rect_width = source_texture.size.width as usize;
    let rect_height = source_texture.size.height as usize;
    let target_width = target_texture.size.width as usize;
    let format_size = target_texture.format.pixel_size();

    for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
        let begin = (bound_y * target_width + rect_x) * format_size;
        let end = begin + rect_width * format_size;
        let texture_begin = texture_y * rect_width * format_size;
        let texture_end = texture_begin + rect_width * format_size;
        target_texture.data[begin..end]
            .copy_from_slice(&source_texture.data[texture_begin..texture_end]);
    }
}

/*----------------------------------------------------------------------------*/

/// Creates the Sprite that shows our assembled map.
///
/// This system gets Commands, which is a queue which can be used to spawn or
/// remove Elements from the World, which is basically the container for
/// everything in a Bevy game. (Where does this "World" come from?
/// We don't need to set it up; it is created as part of the App in main,
/// below.)
///
fn create_map_sprite_system(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mapengine_map: Res<MapEngineMap>,
) {
    // The resource MapEngineMap should already be defined, including
    // a tiny empty texture.
    // This line does two things: adds that texture as a global resource,
    // and also gets us a handle to put into the SpriteBundle as a material.
    // Bevy needs both of these things in order to actually render.
    let map_texture_handle = textures.add(mapengine_map.texture.clone());

    // And here is our "sprite" which shows the whole map. I use "sprite"
    // in scare quotes because it might be quite a bit larger than what
    // that name normally implies, but, hey, we work with what we have.
    // We add the MapEngineSprite component so we can keep this straight
    // from any other sprites.
    commands
        .spawn(SpriteBundle {
            material: materials.add(map_texture_handle.into()),
            ..Default::default()
        })
        .with(MapEngineSprite);
}

/// Draw spaces that need updated onto the map texture.
///
/// TODO Handle removal of spaces, not just addition
///
/// This runs every frame when the engine is in the Running state, so it
/// is important to not do slow things. Unfortunately, because Bevy
/// does not yet support GPU texture-to-texture copy or batched rendering,
/// there are more slow operations here than ideal.
///
/// The first Query here returns MapSpace entities that have MapSpaceRefreshNeeded
/// And the second one gets us all of our map sprites. (There might be
/// more than one for a different view into the same map; that's to be implemented.)
fn maptexture_update_system(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mapengine_map: ResMut<MapEngineMap>,
    mapspaces: Query<(Entity, &MapSpace), With<MapSpaceRefreshNeeded>>,
    mapsprites: Query<&Handle<ColorMaterial>, With<MapEngineSprite>>,
) {
    // MapSpaces are entities in the World. They should be tagged
    // with MapSpaceRefreshNeeded if they've changed in appearance,
    // which will cause this system to get them.

    // This first pass gathers information needed to size the map texture,
    // and if we need to do anything at all.
    // TODO This doubles the number of times we go through the list;
    // consider if it is really the best way. (One idea for an alternate
    // approach: check the map size when spawning a new mapspace, and
    // mark it to grow if need be then.)
    let mut count = 0;
    for (_entity, mapspace) in mapspaces.iter() {
        // Find the furthest-from 0,0 rows and columns.
        // The +1 is because we are zero-indexed, so if everything is in col 0
        // we still need a space_width-wide map.
        mapengine_map.cols = cmp::max(mapengine_map.cols, mapspace.col + 1);
        mapengine_map.rows = cmp::max(mapengine_map.rows, mapspace.row + 1);
        count += 1;
    }
    // If there aren't any, exit now.
    // TODO Refactor so this happens instantly at the beginning of the system
    if count == 0 {
        return;
    }

    // We need to copy these out of the resource because later there's
    // a mutable+immutable borrow attempt if we don't have our own copy.
    let space_width_pixels = mapengine_map.space_width_pixels;
    let space_height_pixels = mapengine_map.space_height_pixels;

    // If our existing texture is too small, create a new bigger one.
    if mapengine_map.texture.size.width < mapengine_map.cols as u32 * space_width_pixels as u32
        || mapengine_map.texture.size.height
            < mapengine_map.rows as u32 * space_height_pixels as u32
    {
        println!(
            "Resizing map texture from {:?}×{:?} to {:?}×{:?}.",
            mapengine_map.texture.size.width,
            mapengine_map.texture.size.height,
            mapengine_map.cols as u32 * space_width_pixels as u32,
            mapengine_map.rows as u32 * space_height_pixels as u32,
        );
        let mut new_texture = Texture::new_fill(
            Extent3d::new(
                mapengine_map.cols as u32 * space_width_pixels as u32,
                mapengine_map.rows as u32 * space_height_pixels as u32,
                1,
            ),
            TextureDimension::D2,
            // transparent
            // FUTURE make this configurable
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
        );

        // copy the old texture to the new one — 0,0 for top left
        copy_texture(&mut new_texture, &mapengine_map.texture, 0, 0);

        // and swap it in.
        mapengine_map.texture = new_texture;
    }

    // And now we iterate through again and do the actual copying
    for (entity, mapspace) in mapspaces.iter() {
        // Each space has a handle to the texture which should represent it visually
        match textures.get(&mapspace.texture_handle) {
            Some(space_texture) => {
                copy_texture(
                    &mut mapengine_map.texture,
                    &space_texture,
                    mapspace.col as usize * space_width_pixels,
                    mapspace.row as usize * space_height_pixels,
                );
            }
            None => {
                eprintln!("For some reason, a texture is missing.");
                std::process::exit(2);
            }
        };
        commands.remove_one::<MapSpaceRefreshNeeded>(entity);
    }

    // As above, this does two things: gets us the handle to put into the
    // sprite, and also adds the texture as a global resource. Bevy
    // needs both of these things to happen in order to actually render.
    let map_texture_handle = textures.add(mapengine_map.texture.clone());

    // We only need to grab the first map sprite, because they
    // all share the same material. And if there isn't one, that's fine;
    // we'll update it once there is in a future pass.
    if let Some(material) = mapsprites.iter().next() {
        materials.get_mut(material).unwrap().texture = Some(map_texture_handle);
    };
}

/*----------------------------------------------------------------------------*/

pub struct MapEnginePlugin;

impl Plugin for MapEnginePlugin {
    fn build(&self, app: &mut AppBuilder) {
        // A stash of handles to our image tiles, so we can use them everywhere.
        app.init_resource::<MapEngineTileHandles>()
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
                create_map_sprite_system.system(),
            )
            // This system runs every frame once we are in the Running state.
            // Because it happens all the time, it needs to be careful to not
            // do slow things. See the code in the maptexture_update_system itself.
            .on_state_update(
                MAPENGINE_STAGE,
                MapEngineState::Running,
                maptexture_update_system.system(),
            );
        // FUTURE add a validator which runs periodically and checks for overlapping MapSpaces?
        // NEXT add a system which takes mouse events and translates them into new events that
        // correspond to the mapspace location (enter, exit, click -- maybe motion?)
        // NEXT possibly also a global resource for current hovered or selected mapspace entity?
        // FUTURE map scrolling (with the mouse stuff still working!)
        // FUTURE map zooming (with the mouse stuff still working!)
    }
}
