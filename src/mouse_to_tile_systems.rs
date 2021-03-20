///  This takes mouse events and outputs our custom tilemouse events.
///
/*----------------------------------------------------------------------------*/
//

// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// We only care about clicks from the mouse events, because position
// comes from window events.
use bevy::input::mouse::MouseButtonInput;

// MouseMotion gives a delta; this gives the cursor position
// TODO deal with CursorLeft as well.
use bevy::window::CursorMoved;

// Used to find the camera
use bevy::render::camera::{Camera, OrthographicProjection};

// Import all of the event structs directly
use crate::tile_mouse_events::*;

/// Used internally to keep our EventReaders' state
#[derive(Default)]
pub struct EventReaderState {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

// TODO need a Local resource to hold the last-seen camera
// state, so we know if we've panned or zoomed since then.

/// This system gets Commands, which is a queue which can be used to spawn or
/// remove Elements from the World, which is basically the container for
/// everything in a Bevy game. (Where does this "World" come from?
/// We don't need to set it up; it is created as part of the App in main,
/// below.)
///
/// This is adapted from https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn mouse_to_tile_system(
    mut eventreaders: Local<EventReaderState>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    query_camera: Query<
        &Transform,
        (
            With<Camera>,
            With<OrthographicProjection>,
            With<crate::MapEngineCamera>,
        ),
    >,
    //commands: &mut Commands,
) {
    //
    //
    // FUTURE Handle multiple map cameras (possibly for a mini-map, etc)
    let camera_transform = query_camera.iter().next().unwrap();

    for event in eventreaders
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {
        //println!("{:?}", event);
    }
    for event in eventreaders
        .cursor_moved_event_reader
        .iter(&cursor_moved_events)
    {
        // Get the size of the window this event is for.
        let window = windows.get(event.id).unwrap();
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // The default orthographic projection is in pixels from the center;
        // This just undoes that translation.=
        let p = event.position - window_size / 2.0;

        println!("{:?} -> {:?}", event, p);
    }
}
