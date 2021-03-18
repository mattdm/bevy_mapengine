/// This is a demo of all of the functionality of this library.
/// Eventually, there will be other, smaller examples for specific
/// features. But we don't have any features yet, so here we are.
///
// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// In order to start full-screen (or not). We will eventually want this.
use bevy::window::WindowMode;

// Built-in Bevy plugins to print FPS to console.
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};

// Until we have our own keyboard handling, this is handy...
use bevy::input::system::exit_on_esc_system;

// These are used for creating the map texture
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

// Used to tell if assets are loaded ... see check_tiles_loaded_system()
use bevy::asset::LoadState;

// Standard rust things...
use rand::Rng;
use std::cmp;

/*----------------------------------------------------------------------------*/

/// Bevy does "lazy" loading of assets. We switch from the
/// Loading state to Running state when all of the tile images
/// are actually loaded.
#[derive(Clone)]
enum MapEngineState {
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
/// TODO consider making col and row read-only using the readonly crate
/// TODO make texture_handle a Vec, and draw in order?
/// The other layering approach (adding depth, allowing multiple col,row)
/// has the disadvantage that we need to find all of the entities to draw.
#[derive(Debug)]
struct MapCell {
    /// Column (x) position of this tile on the map. 0 is on the left.
    col: i32,
    /// Row (y) position of this tile on the map. 0 is at the top.
    row: i32,
    /// load with, for example, `asset_server.get_handle("terrain/grass1.png")`
    texture_handle: Handle<Texture>,
}

/// This component signals that a mapcell needs to be refreshed.
/// This is a hack until https://github.com/bevyengine/bevy/pull/1471 is implemented.
struct MapCellRefreshNeeded;

/// Bevy groups systems into stages. Our mapengine
/// runs in its own stage, and this is its name.
/// See main() for how this is actually used.
const MAPENGINE_STAGE: &str = "mapengine_stage";

/// Our list of handles to tile images is stored as a global
/// Bevy resource so we can use them in various systems. In Bevy,
/// these global resources are located by type, so we need a custom
/// type to do this.
#[derive(Default)]
struct MapEngineTileHandles {
    handles: Vec<HandleUntyped>,
}

/// This is for the global resource that holds our map information.
struct MapEngineMap {
    /// The actual texture to be drawn on
    texture: Texture,
    /// Width of map in cells (texture width = cols × cell_width_pixels)
    cols: i32,
    /// Height of map in cells (texture height = rows × cell_height_pixels)
    rows: i32,
    /// Each cell must be the same; keeping it here saves us reading it later.
    cell_width_pixels: usize,
    /// Each cell must be the same; keeping it here saves us reading it later.
    cell_height_pixels: usize,
}

/// This component tags a sprite as map sprite
struct MapEngineSprite;

impl Default for MapEngineMap {
    /// default to an empty texture
    fn default() -> Self {
        MapEngineMap {
            // We start with the minimum possible texture size: 1×1
            // TODO have a reasonable default and make configurable
            texture: Texture::new_fill(
                Extent3d::new(1, 1, 1),
                TextureDimension::D2,
                &[0, 0, 0, 0],
                TextureFormat::Rgba8UnormSrgb,
            ),
            cols: 0,
            rows: 0,
            cell_width_pixels: 0,
            cell_height_pixels: 0,
        }
    }
}

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

/// This function is a "system" — see the App builder in main(), below.
/// It is configured there to run once at the beginning of the initial
/// "state", which we have named "Loading". (See the MapEngineState enum.)
///
/// Part of Bevy's magic is that the app is generated such that system
/// like this one get "fed" various information automatically based on
/// the parameters you give it. Here, we are getting two different
/// global Res(ources): an AssetServer and the collection of tile handles
/// we defined above (and initialize as a resource in main()).
fn load_tiles_system(
    asset_server: Res<AssetServer>,
    mut tilehandles: ResMut<MapEngineTileHandles>,
) {
    // The asset server defaults to looking in the `assets` directory.
    // This call loads everything in the `terrain` subfolder as our
    // tile images and stores the list of handles in the global resource.
    match asset_server.load_folder("terrain") {
        Ok(handles) => tilehandles.handles = handles,
        Err(err) => {
            eprintln!("Error: Problem loading terrain textures ({:?})", err);
            std::process::exit(1);
        }
    }
}

/// This system is configured to run as part of the game loop while in
/// the "Loading" state. It checks if the various tile handles are
/// all actually available, and advances the state if they are.
///
/// Here, you can see that in addition to the resources the load system
/// uses we also get the State resource. And since we don't modify the
/// tilehandles here, that resource is not mutable.
fn wait_for_tile_load_system(
    mut state: ResMut<State<MapEngineState>>,
    tilehandles: ResMut<MapEngineTileHandles>,
    asset_server: Res<AssetServer>,
) {
    // Note that this is pretty much always going to be "NotLoaded" until it becomes "Loaded".
    // The "Loading" state is unlikely because get_group_load_state returns not loaded if _any_ are.
    match asset_server.get_group_load_state(tilehandles.handles.iter().map(|handle| handle.id)) {
        LoadState::NotLoaded => println!("Loading terrain textures..."),
        LoadState::Loading => println!("Loading terrain textures..."),
        LoadState::Loaded => {
            println!("Terrain textures loaded!");
            // Finally advance the State
            state.set_next(MapEngineState::Verifying).unwrap();
        }
        LoadState::Failed => {
            eprintln!("Failed to load terrain textures!");
            std::process::exit(1)
        }
    }
}

/// A system which shecs to make sure all of the loaded tiles are
/// valid and then advances to the next game State (Running).
/// A more advanced version of this could go to an Error state
/// instead of existing on failure. Combined with dynamic asset
/// loading, that could make it possible to actually fix the
/// problem and resume.
fn verify_tiles_system(
    mut state: ResMut<State<MapEngineState>>,
    tilehandles: ResMut<MapEngineTileHandles>,
    textures: Res<Assets<Texture>>,
    mut mapengine_map: ResMut<MapEngineMap>,
) {
    // This crazy code does this:
    //
    // 1. Gets the widths, heights, and depths of all textures
    // 2. Sets mapengine_map height and width
    // 3. Errors if any tiles are differently-sized
    // 4. Errors if any depth is anything but 1
    // 5. And if all that succeeds, moves on to Running
    //
    // We could add other verification here as well, of course.
    let widths = tilehandles
        .handles
        .iter()
        .map(|handle| textures.get(handle).unwrap().size.width)
        .collect::<Vec<u32>>();
    let heights = tilehandles
        .handles
        .iter()
        .map(|handle| textures.get(handle).unwrap().size.height)
        .collect::<Vec<u32>>();
    let depths = tilehandles
        .handles
        .iter()
        .map(|handle| textures.get(handle).unwrap().size.depth)
        .collect::<Vec<u32>>();

    mapengine_map.cell_width_pixels = widths[0] as usize;
    mapengine_map.cell_height_pixels = widths[0] as usize;

    if widths
        .iter()
        .any(|&w| w != mapengine_map.cell_width_pixels as u32)
    {
        eprintln!("Error! All tile textures must be the same width (at least one isn't).");
        std::process::exit(1)
    }
    if heights
        .iter()
        .any(|&h| h != mapengine_map.cell_height_pixels as u32)
    {
        eprintln!("Error! All tile textures must be the same height (at least one isn't).");
        std::process::exit(1)
    }
    if depths.iter().any(|&d| d != 1) {
        eprintln!("Error! At least of the tile textures isn't two-dimensional!");
        std::process::exit(1)
    }

    println!(
        "{:?} terrain textures of size {:?}×{:?} found.",
        widths.len(),
        mapengine_map.cell_width_pixels,
        mapengine_map.cell_height_pixels
    );

    state.set_next(MapEngineState::Running).unwrap();
}

/// A very simple system which just makes it so we can see the world.
///
/// This gets Commands, which is a queue which can be used to spawn or
/// remove Elements from the World, which is basically the container for
/// everything in a Bevy game. (Where does this "World" come from?
/// We don't need to set it up; it is created as part of the App in main,
/// below.)
///
fn setup_camera_system(commands: &mut Commands) {
    // This sets up the default 2d camera, which has an orthgraphic (staight ahead,
    // everything square-on) view.
    commands.spawn(Camera2dBundle::default());
}

/// This is a one-time system that spawns some MapCell components.
/// For a future phase of this demo we'll need something more sophisticated,
/// but this works for now. It needs Commands to do the spawning, and the
/// AssetServer resource to get the handles for textures by name.
/// TODO Maybe parse a text file or multi-line string with character
/// representations of the map?
fn setup_demo_map_system(commands: &mut Commands, asset_server: Res<AssetServer>) {
    // We're going to put down a bunch of stuff at random, so we
    // will need a random number generator.
    let mut rng = rand::thread_rng();

    for row in 0..12 {
        for col in 0..20 {
            // Most likely to just be grass, but throw in some
            // trees as well.
            let tile_type = match rng.gen_range(0..50) {
                0 => "terrain/tree6.png",
                1 => "terrain/pine6.png",
                2..=3 => "terrain/tree3.png",
                4..=5 => "terrain/pine3.png",
                6..=8 => "terrain/tree2.png",
                9..=11 => "terrain/pine2.png",
                12..=15 => "terrain/tree2.png",
                16..=19 => "terrain/pine2.png",
                20..=30 => "terrain/grass1.png",
                _ => "terrain/grass2.png",
            };

            commands
                .spawn((MapCell {
                    col: col,
                    row: row,
                    texture_handle: asset_server.get_handle(tile_type),
                },))
                .with(MapCellRefreshNeeded);
        }
    }
}

/// Creates the sprite that shows our assembled map.
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

/// Draw cells that need updated onto the map texture.
///
/// TODO Handle removal of cells, not just addition
///
/// This runs every frame when the engine is in the Running state, so it
/// is important to not do slow things. Unfortunately, because Bevy
/// does not yet support GPU texture-to-texture copy or batched rendering,
/// there are more slow operations here than ideal.
///
/// The first Query here returns MapCell entities that have MapCellRefreshNeeded
/// And the second one gets us all of our map sprites. (There might be
/// more than one for a different view into the same map; that's to be implemented.)
fn maptexture_update_system(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mapengine_map: ResMut<MapEngineMap>,
    mapcells: Query<(Entity, &MapCell), With<MapCellRefreshNeeded>>,
    mapsprites: Query<&Handle<ColorMaterial>, With<MapEngineSprite>>,
) {
    // MapCells are entities in the World. They should be tagged
    // with MapCellRefreshNeeded if they've changed in appearance,
    // which will cause this system to get them.

    // FIXME refactor to be a no-op if mapcells is empty
    // (without needing to go through them an extra time when it isn't!)

    // This first pass gathers information needed to size the map texture.
    // TODO This doubles the number of times we go through the list;
    // consider if it is really the best way.
    for (_entity, mapcell) in mapcells.iter() {
        // Find the furthest-from 0,0 rows and columns.
        // The +1 is because we are zero-indexed, so if everything is in col 0
        // we still need a cell_width-wide map.
        mapengine_map.cols = cmp::max(mapengine_map.cols, mapcell.col + 1);
        mapengine_map.rows = cmp::max(mapengine_map.rows, mapcell.row + 1);
    }

    // We need to copy these out of the resource because later there's
    // a mutable+immutable borrow attempt if we don't have our own copy.
    let cell_width_pixels = mapengine_map.cell_width_pixels;
    let cell_height_pixels = mapengine_map.cell_height_pixels;

    // If our existing texture is too small, create a new bigger one.
    if mapengine_map.texture.size.width < mapengine_map.cols as u32 * cell_width_pixels as u32
        || mapengine_map.texture.size.height < mapengine_map.rows as u32 * cell_height_pixels as u32
    {
        println!(
            "Resizing map texture from {:?}×{:?} to {:?}×{:?}.",
            mapengine_map.texture.size.width,
            mapengine_map.texture.size.height,
            mapengine_map.cols as u32 * cell_width_pixels as u32,
            mapengine_map.rows as u32 * cell_height_pixels as u32,
        );
        let mut new_texture = Texture::new_fill(
            Extent3d::new(
                mapengine_map.cols as u32 * cell_width_pixels as u32,
                mapengine_map.rows as u32 * cell_height_pixels as u32,
                1,
            ),
            TextureDimension::D2,
            // transparent
            // TODO make this configurable
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
        );

        // copy the old texture to the new one — 0,0 for top left
        copy_texture(&mut new_texture, &mapengine_map.texture, 0, 0);

        // and swap it in.
        mapengine_map.texture = new_texture;
    }

    // And now we iterate through again and do the actual copying
    for (entity, mapcell) in mapcells.iter() {
        // Each cell has a handle to the texture which should represent it visually
        match textures.get(&mapcell.texture_handle) {
            Some(cell_texture) => {
                copy_texture(
                    &mut mapengine_map.texture,
                    &cell_texture,
                    mapcell.col as usize * cell_width_pixels,
                    mapcell.row as usize * cell_height_pixels,
                );
            }
            None => {
                eprintln!("For some reason, a texture is missing.");
                std::process::exit(2);
            }
        };
        commands.remove_one::<MapCellRefreshNeeded>(entity);
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

fn main() {
    App::build()
        // The window is created by WindowPlugin. This is a global resource
        // which that plugin looks for to find its configuration. This is a
        // common Bevy pattern for configuring plugins.
        .add_resource(WindowDescriptor {
            title: "Bevy Mapengine Demo".to_string(),
            width: 1280.,
            height: 720.,
            vsync: true,
            resizable: false, // todo: cope with resizable windows
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        // This sets up all the basic Bevy engine stuff. Basically,
        // nothing in Bevy will work without 90% of this, so most people
        // just include it all. Note that for this project, audio and
        // gltf (a 3d graphic format) are disabled in Cargo.toml.
        .add_plugins(DefaultPlugins)
        // These two collect and print frame count statistics to the console
        // TODO add a command line option to turn these two on or off instead of messing with comments
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        // This is a built-in-to-Bevy handy keyboard exit function
        .add_system(exit_on_esc_system.system())
        // Now, we are finally on to our own code — that is, stuff here in this demo.
        // The first system is really simple: it sets up a camera. It is a _startup system_,
        // which means it only runs once at the beginning, before everything else.
        // This won't be part of our plugin -- it'll be expected that the game using our
        // plugin will do this.
        .add_startup_system(setup_camera_system.system())
        // This inserts MapCell entities from which the map will be built.
        .add_startup_system(setup_demo_map_system.system())
        // A stash of handles to our image tiles, so we can use them everywhere.
        .init_resource::<MapEngineTileHandles>()
        // This adds a "Stage" (basically, a group of systems) set up to handle our
        // various "States". Our stage, used in the MapEngine, will run right after
        // the default UPDATE stage. This is important because otherwise we will miss
        // changes to MapCell entities done in the plugin user's code.
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
            load_tiles_system.system(),
        )
        // And this stage runs every frame while still in Loading state
        // (and is responsible for changing the state to Checking when ready)
        .on_state_update(
            MAPENGINE_STAGE,
            MapEngineState::Loading,
            wait_for_tile_load_system.system(),
        )
        // This stage makes sure that our tiles are valid and stores information
        // about them in the global MapEngineMap resource, and then advances
        // the state to Running. It exits on failure; we could get even more
        // fancy and instead have an Error state which presents error messages in-game.
        .on_state_enter(
            MAPENGINE_STAGE,
            MapEngineState::Verifying,
            verify_tiles_system.system(),
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
        )
        // TODO add a validator which runs periodically and checks for overlapping MapCells?
        // TODO add a system which takes mouse events and translates them into new events that
        // correspond to the mapcell location (enter, exit, click -- maybe motion?)
        // TODO possibly also a global resource for current hovered or selected mapcell entity?
        //
        // And finally, this, which fires off the actual game loop.
        .run()
}
