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

/// We want to store our list of handles to tile images as a global
/// Bevy resource. In Bevy, these global resources are located by type,
/// so we need a custom type to do this.
#[derive(Default)]
struct MapEngineTileHandles {
    handles: Vec<HandleUntyped>,
}

/// This function is called as a Bevy startup function — see the App
/// builder in main, below. The name `setup` is not magical, but it's
/// a straightforward-enough convention.
///
/// Part of Bevy's magic is that the app is generated such that system
/// like this one get "fed" various information automatically based on
/// the parameters you give it. Here, we are getting Commands, which
/// be used to spawn or remove Elements from the World, plus several
/// global Res(ources): an AssetServer and collections of Assets of
/// type Texture and TextureAtlas.
///
/// The AssetServer is used to load a texture from our on-disk
/// sample tilemap image, and the Asset collections store handles to the
/// loaded texture and a list of rectangular areas within that texture
/// which can be used for individual Sprites. Other systems might use
/// Query to get access to selected Entities stored in the World.
///
/// (Where does this "World" come from? We don't need to set it up; it is
/// created as part of the App in main, below.)
fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tilehandles: ResMut<MapEngineTileHandles>,
) {
    // This sets up the default 2d camera, which has an orthgraphic (staight ahead,
    // everything square-on) view.
    commands.spawn(Camera2dBundle::default());

    // The asset server defaults to looking in the `assets` directory.
    // This function loads everything in the `terrain` subfolder as our
    // tile images and stores the list of handles in the global resource.
    // FIXME remove the unwrap and handle errors properly!
    // FIXME check if assets are actually loaded before moving on
    tilehandles.handles = asset_server.load_folder("terrain").unwrap();

    // Here we create a shiny new empty texture which will serve as
    // the "canvas" for our world map.
    //
    // Temporarily, this is bright red so we can see that it's working.
    let map_texture = textures.add(Texture::new_fill(
        Extent3d::new(1280, 1280, 1),
        TextureDimension::D2,
        &[255, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
    ));

    // For testing, we create a sprite which shows the whole big texture
    commands.spawn(SpriteBundle {
        material: materials.add(map_texture.into()),
        ..Default::default()
    });

    // And another test sprite
    commands.spawn(SpriteBundle {
        material: materials.add(asset_server.get_handle("terrain/grass1.png").into()),
        ..Default::default()
    });

    // And now for something horrible. Bevy does not yet have a way to
    // actually copy/draw from texture to texture. So we are going to do
    // it the hard way.

    // And now we create a grid of Entities...
}

//fn testing(mut query: Query<&mut Texture>) {}

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
        .init_resource::<MapEngineTileHandles>()
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
        // This is a added as a "startup system", which runs only once at the beginning.
        .add_startup_system(setup.system())
        // for testing, of course
        //.add_system(testing.system())
        // And this, of course, fires off the actual game loop.
        .run()
}
