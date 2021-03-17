/// This is a demo of all of the functionality of this library.
/// Eventually, there will be other, smaller examples for specific
/// features. But we don't have any features yet, so here we are.
///
// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// In order to start full-screen (or not). We will eventually want this.
use bevy::window::WindowMode;

// Built-in Bevy plugins to print FPS to console.
//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};

// Until we have our own keyboard handling, this is handy...
use bevy::input::system::exit_on_esc_system;

// These are used for creating the map texture
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

// Used to tell if assets are loaded ... see check_tiles_loaded_system()
use bevy::asset::LoadState;

/*----------------------------------------------------------------------------*/

/// Bevy does "lazy" loading of assets. We switch from the
/// Loading state to Running state when all of the tile images
/// are actually loaded.
#[derive(Clone)]
enum MapEngineState {
    Loading,
    Running,
}

/// This is a Bevy Component that defines an Entity as representing
/// a space on our map, and holds the location and the tile image
/// to use. Note that these are meant to represent fixed locations
/// on the map; x and y should not change. Note also that I haven't
/// decided on how to do layering, so duplicating (col,row) will
/// lead to unpredictable results.
#[derive(Debug)]
struct MapCell {
    /// Column (x) position of this tile on the map. 0 is the center of the world.
    col: usize,
    /// Row (y) position of this tile on the map. 0 is the center of the world.
    row: usize,
    /// load with, for example, `asset_server.get_handle("terrain/grass1.png")`
    texture_handle: Handle<Texture>,
}

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
    /// Each cell must be the same; keeping it here saves us reading it later.
    cell_width: usize,
    /// Each cell must be the same; keeping it here saves us reading it later.
    cell_height: usize,
}

impl Default for MapEngineMap {
    fn default() -> Self {
        MapEngineMap {
            texture: Texture::default(),
            cell_width: 64,
            cell_height: 64,
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
fn copy_texture(target_texture: &mut Texture, texture: &Texture, rect_x: usize, rect_y: usize) {
    let rect_width = texture.size.width as usize;
    let rect_height = texture.size.height as usize;
    let target_width = target_texture.size.width as usize;
    let format_size = target_texture.format.pixel_size();

    for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
        let begin = (bound_y * target_width + rect_x) * format_size;
        let end = begin + rect_width * format_size;
        let texture_begin = texture_y * rect_width * format_size;
        let texture_end = texture_begin + rect_width * format_size;
        target_texture.data[begin..end].copy_from_slice(&texture.data[texture_begin..texture_end]);
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
    // FIXME remove the unwrap and handle errors properly!
    tilehandles.handles = asset_server.load_folder("terrain").unwrap();
}

/// This system is configured to run as part of the game loop while in
/// the "Loading" state. It checks if the various tile handles are
/// all actually available, and advances the state if they are.
///
/// Here, you can see that in addition to the resources the load system
/// uses we also get the State resource. And since we don't modify the
/// tilehandles here, it's not mutable.
///
/// This function is adapted pretty directly from the `texture_atlas.rs`
/// example in Bevy 0.4.0.
fn wait_for_tile_load_system(
    mut state: ResMut<State<MapEngineState>>,
    tilehandles: ResMut<MapEngineTileHandles>,
    asset_server: Res<AssetServer>,
) {
    // FIXME Expand into a match and show the different LoadStates
    if let LoadState::Loaded =
        asset_server.get_group_load_state(tilehandles.handles.iter().map(|handle| handle.id))
    {
        state.set_next(MapEngineState::Running).unwrap();
    }
    // FIXME verify that all of the tiles are the same size and write the dimension to the MapEngineMap resource.
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
fn setup_demo_map_system(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands.spawn((MapCell {
        col: 0,
        row: 0,
        texture_handle: asset_server.get_handle("terrain/pine6.png"),
    },));
}

/// This is a playground for creating the map texture
fn maptexture_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mapengine_map: ResMut<MapEngineMap>,
    mapcells: Query<(&MapCell)>,
) {
    // Here we create a shiny new empty texture which will serve as
    // the "canvas" for our world map.
    //
    // Temporarily, this is bright red so we can see that it's working.
    mapengine_map.texture = Texture::new_fill(
        Extent3d::new(720, 720, 1),
        TextureDimension::D2,
        &[255, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
    );

    for mapcell in mapcells.iter() {
        println!("{:#?}", mapcell);

        // FIXME handle missing textures instead of unwrap!
        let cell_texture = textures.get(&mapcell.texture_handle).unwrap();
        // FIXME location, location, location!
        copy_texture(&mut mapengine_map.texture, &cell_texture, 0, 0);
    }

    let map_texture_handle = textures.add(mapengine_map.texture.clone());

    // This "sprite" shows the whole map.
    commands.spawn(SpriteBundle {
        material: materials.add(map_texture_handle.into()),
        ..Default::default()
    });
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
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.add_plugin(PrintDiagnosticsPlugin::default())
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
        // the default UPDATE stage. See https://bevy-cheatbook.github.io/basics/stages.html
        // for more on stages.
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
        // (and is responsible for changing the state to Running when ready)
        .on_state_update(
            MAPENGINE_STAGE,
            MapEngineState::Loading,
            wait_for_tile_load_system.system(),
        )
        // This system will run once when we get to the Running state.
        // It's a temporary thing, because eventually we want a system which
        // runs every frame looking for changed MapCell entities.
        .on_state_enter(
            MAPENGINE_STAGE,
            MapEngineState::Running,
            maptexture_system.system(),
        )
        // And finally, this, which fires off the actual game loop.
        .run()
}
