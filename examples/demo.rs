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
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};

// Until we have our own keyboard handling, this is handy...
use bevy::input::system::exit_on_esc_system;

fn main() {
    App::build()
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
        // These two collect and print frame count statistics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        .add_system(exit_on_esc_system.system())
        .run()
}
