/// This is a demo of all of the functionality of this library.
/// Eventually, there will be other, smaller examples for specific
/// features. But we don't have any features yet, so here we are.
///
/*----------------------------------------------------------------------------*/
//

// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// In order to start full-screen (or not). We will eventually want this.
use bevy::window::WindowMode;

// Built-in Bevy plugins to print FPS to console.
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};

// Until we have our own keyboard handling, this is handy...
use bevy::input::system::exit_on_esc_system;

// Standard rust things...
use rand::Rng;

// This is ... the thing being demonstrated here :)
use bevy_mapengine::{MapEngineConfig, MapEnginePlugin, MapSpace, MapSpaceRefreshNeeded};

/*----------------------------------------------------------------------------*/

/// A very simple system which just makes it so we can see the world.
fn setup_camera_system(commands: &mut Commands) {
    // This sets up the default 2d camera, which has an orthgraphic (staight ahead,
    // everything square-on) view.
    commands.spawn(Camera2dBundle::default());
}

/// This is a one-time system that spawns some MapSpace components.
/// For a future phase of this demo we'll need something more sophisticated,
/// but this works for now. It needs Commands to do the spawning, and the
/// AssetServer resource to get the handles for textures by name.
// FUTURE Maybe parse a text file or multi-line string with character
// representations of the map?
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

            // TODO Don't spawn MapSpace entities directly, but rather request for their creation.
            commands
                .spawn((MapSpace {
                    col: col,
                    row: row,
                    texture_handle: asset_server.get_handle(tile_type),
                },))
                .with(MapSpaceRefreshNeeded);
        }
    }
}

/*----------------------------------------------------------------------------*/

fn main() {
    App::build()
        // The window is created by WindowPlugin. This is a global resource
        // which that plugin looks for to find its configuration. This is a
        // common Bevy pattern for configuring plugins.
        .add_resource(WindowDescriptor {
            title: "Bevy MapEngine Demo".to_string(),
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
        // FUTURE add a command line option to turn these two on or off instead of messing with comments
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        // This is a built-in-to-Bevy handy keyboard exit function
        .add_system(exit_on_esc_system.system())
        // This resource gives the configuration for the MapEngine plugin.
        // In specific, it tells the folder to load the terrain tiles from.
        .add_resource(MapEngineConfig {
            // NEXT adapt library code so it can take either a str or String
            tile_folder: "terrain".to_string(),
        })
        // And this is the MapEngine plugin — it loads all the systems
        // which handle putting entities with the MapSpace component
        // onto the actual map.
        .add_plugin(MapEnginePlugin)
        // Now, we are finally on to our own code — that is, stuff here in this demo.
        // The first system is really simple: it sets up a camera. It is a _startup system_,
        // which means it only runs once at the beginning, before everything else.
        // This won't be part of our plugin — it'll be expected that the game using our
        // plugin will do this.
        .add_startup_system(setup_camera_system.system())
        // This inserts MapSpace entities from which the map will be built.
        .add_startup_system(setup_demo_map_system.system())
        // And finally, this, which fires off the actual game loop.
        .run()
}
