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
pub fn mouse_to_tile_system(
    mut eventreaders: Local<EventReaderState>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    //commands: &mut Commands,
) {
    for event in eventreaders
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {
        println!("{:?}", event);
    }
    for event in eventreaders
        .cursor_moved_event_reader
        .iter(&cursor_moved_events)
    {
        println!("{:?}", event);
    }
}
