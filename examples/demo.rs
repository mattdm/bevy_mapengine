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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // This sets up the default 2d camera, which has an orthgraphic (staight ahead,
    // everything square-on) view.
    commands.spawn(Camera2dBundle::default());

    // The asset server defaults to looking in the `assets` directory. You
    // can do some fancy things like automatically loading changes from disk,
    // but we're not going for any of that here (yet, at least).
    let demo_tilesheet_handle = asset_server.load("medieval_tilesheet.png");

    // We're loading our texture_atlas with an image which happens to have
    // 128×128 pixel tiles with 64 pixels of padding in between. And Bevy
    // has a function to load into a texture map from an image formatted that
    // that way, which is super-handy! ("18" and "7" are columns and rows.)
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        demo_tilesheet_handle,
        Vec2::new(128.0, 128.0),
        18,
        7,
        Vec2::new(64.0, 64.0),
    );

    // This does two things: adds the TextureAtlas we just created
    // to the global Resource, and gets a Handle to that TextureAtlas
    // which we can use later when creating Entities that represent
    // map cells.
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

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

    // And now we create a grid of Entities with SpriteSheetBundle.
    //
    // Bundles are collections of Components, and this bundle has stuff
    // that tells the built-in RenderPlugin (part of DefaultPlugins) to
    // actually draw this thing, and how. Note that this is just Rust —
    // the bundle is a struct, and the default() part fills in the
    // standard stuff for a sprite sheet, plus of course the
    // texture_atlas_handle we are giving it now.
    //
    // We could also tack .with() calls to the end of the spawn command
    // to add additional Components beyond those in the bundle.
    commands.spawn(SpriteSheetBundle {
        // Each sprite holds a pointer to the texture atlas
        texture_atlas: texture_atlas_handle,
        // and also a color (which will shade the render!)
        // and an index (by row then column) of the specific tile
        sprite: TextureAtlasSprite {
            color: Color::WHITE,
            index: 0,
        },
        ..Default::default()
    });
}

fn testing(mut query: Query<&mut Texture>) {}

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
        // And this, of course, fires off the actual game loop.
        .add_system(testing.system())
        .run()
}
