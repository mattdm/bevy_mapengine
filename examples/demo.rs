/// This is a demo of all of the functionality of this library.
/// Eventually, there will be other, smaller examples for specific
/// features. But we don't have any features yet, so here we are.
///
// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// In order to start full-screen (or not). We will eventually want this.
use bevy::window::WindowMode;

// So, ironically, we have to bring into scope all of the plugins we want
// to _disable_ from the defaults. Some of these we will use eventually, but
// I'm leaving them out for now.
use bevy::{audio::AudioPlugin, gltf::GltfPlugin};

// Built-in Bevy plugins to print FPS to console.
//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};

// Until we have our own keyboard handling, this is handy...
use bevy::input::system::exit_on_esc_system;

// These are used for creating the map texture
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

// Used to tell if assets are loaded ... see check_tiles_loaded_system()
use bevy::asset::LoadState;

/// Bevy does "lazy" loading of assets. We switch from the
/// Loading state to Running state when all of the tile images
/// are actually loaded.
#[derive(Clone)]
enum MapEngineState {
    Loading,
    Running,
}

/// Bevy groups systems into stages. Our mapengine
/// runs in its own stage, and this is its name.
/// See main() for how this is actually used.
const MAPENGINE_STAGE: &str = "mapengine_stage";

/// We want to store our list of handles to tile images as a global
/// Bevy resource. In Bevy, these global resources are located by type,
/// so we need a custom type to do this.
#[derive(Default)]
struct MapEngineTileHandles {
    handles: Vec<HandleUntyped>,
}

/// Ripped from bevy_sprite/src/texture_atlas_builder.rs.
///
/// This doesn't really copy actual GPU textures. It copies bits
/// in a Vec representing RGBA data. This is not going way we want
/// to do this always, but we are waiting on
/// https://github.com/bevyengine/bevy/issues/1207#issuecomment-800602680
/// for a real solution.
fn copy_texture(target_texture: &mut Texture, texture: &Texture, rect_x: usize, rect_y: usize) {
    let rect_width = 128 as usize;
    let rect_height = 128 as usize;
    let atlas_width = target_texture.size.width as usize;
    let format_size = target_texture.format.pixel_size();

    for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
        let begin = (bound_y * atlas_width + rect_x) * format_size;
        let end = begin + rect_width * format_size;
        let texture_begin = texture_y * rect_width * format_size;
        let texture_end = texture_begin + rect_width * format_size;
        target_texture.data[begin..end].copy_from_slice(&texture.data[texture_begin..texture_end]);
    }
}

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

/// This is a playground for creating the map texture
fn maptexture_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Here we create a shiny new empty texture which will serve as
    // the "canvas" for our world map.
    //
    // Temporarily, this is bright red so we can see that it's working.
    let mut map_texture = Texture::new_fill(
        Extent3d::new(512, 512, 1),
        TextureDimension::D2,
        &[255, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
    );

    let other_texture_handle: Handle<Texture> = asset_server.get_handle("terrain/grass1.png");
    let other_texture = textures.get(other_texture_handle).unwrap();

    copy_texture(&mut map_texture, &other_texture, 0, 0);

    let map_texture_handle = textures.add(map_texture);
    // For testing, we create a sprite which shows the whole big texture
    commands.spawn(SpriteBundle {
        material: materials.add(map_texture_handle.into()),
        ..Default::default()
    });
}

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
        // nothing in Bevy will work without 90% of this.
        .add_plugins_with(DefaultPlugins, |group| {
            // We're not using audio, and gltf is for 3d scenes.
            group.disable::<AudioPlugin>().disable::<GltfPlugin>()
        })
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
        // This stage happens once when entering the Loading state (that is, right away)
        .on_state_enter(
            MAPENGINE_STAGE,
            MapEngineState::Loading,
            load_tiles_system.system(),
        )
        // and this stage runs every frame while still in Loading state
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
