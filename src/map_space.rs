use bevy::prelude::*;

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
