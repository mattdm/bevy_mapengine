/// This collection of systems for collecting MapSpace entities from the
/// World and drawing them on our map Sprite to be rendered by Bevy.
/*----------------------------------------------------------------------------*/
//

// This is the basic Bevy game engine stuff
use bevy::prelude::*;

// Used to tell if assets are loaded ... see check_tiles_loaded_system()
use bevy::asset::LoadState;

/// Our list of handles to tile images is stored as a global
/// Bevy resource so we can use them in various systems. In Bevy,
/// these global resources are located by type, so we need a custom
/// type to do this.
#[derive(Default)]
pub struct MapEngineTileHandles {
    handles: Vec<HandleUntyped>,
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
pub fn load_tiles_system(
    asset_server: Res<AssetServer>,
    mut tilehandles: ResMut<MapEngineTileHandles>,
) {
    // The asset server defaults to looking in the `assets` directory.
    // This call loads everything in the `terrain` subfolder as our
    // tile images and stores the list of handles in the global resource.
    // FIXME don't hard-code "terrain" here but instead get it when setting up the plugin
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
pub fn wait_for_tile_load_system(
    mut state: ResMut<State<crate::MapEngineState>>,
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
            state.set_next(crate::MapEngineState::Verifying).unwrap();
        }
        LoadState::Failed => {
            eprintln!("Failed to load terrain textures!");
            std::process::exit(1)
        }
    }
}

/// A system which checks to make sure all of the loaded tiles are
/// valid and then advances to the next game State (Running).
/// A more advanced version of this could go to an Error state
/// instead of existing on failure. Combined with dynamic asset
/// loading, that could make it possible to actually fix the
/// problem and resume.
pub fn verify_tiles_system(
    mut state: ResMut<State<crate::MapEngineState>>,
    tilehandles: ResMut<MapEngineTileHandles>,
    textures: Res<Assets<Texture>>,
    mut mapengine_map: ResMut<crate::map::Map>,
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

    mapengine_map.space_width_pixels = widths[0] as usize;
    mapengine_map.space_height_pixels = widths[0] as usize;

    if widths
        .iter()
        .any(|&w| w != mapengine_map.space_width_pixels as u32)
    {
        eprintln!("Error! All tile textures must be the same width (at least one isn't).");
        std::process::exit(1)
    }
    if heights
        .iter()
        .any(|&h| h != mapengine_map.space_height_pixels as u32)
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
        mapengine_map.space_width_pixels,
        mapengine_map.space_height_pixels
    );

    state.set_next(crate::MapEngineState::Running).unwrap();
}
