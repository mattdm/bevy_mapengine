/// Ripped from bevy_sprite/src/texture_atlas_builder.rs.
///
/// This doesn't really copy actual GPU textures. It copies bits
/// in a Vec representing RGBA data. This is not going way we want
/// to do this always, but we are waiting on
/// https://github.com/bevyengine/bevy/issues/1207#issuecomment-800602680
/// for a real solution.
///
///
//

// This is the basic Bevy game engine stuff
use bevy::prelude::*;
// These are used for creating the map texture
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

// Standard rust things...
use std::cmp;

fn copy_texture(
    target_texture: &mut Texture,
    source_texture: &Texture,
    rect_x: usize,
    rect_y: usize,
) {
    let rect_width = source_texture.size.width as usize;
    let rect_height = source_texture.size.height as usize;
    let target_width = target_texture.size.width as usize;
    let format_size = target_texture.format.pixel_size();

    for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
        let begin = (bound_y * target_width + rect_x) * format_size;
        let end = begin + rect_width * format_size;
        let texture_begin = texture_y * rect_width * format_size;
        let texture_end = texture_begin + rect_width * format_size;
        target_texture.data[begin..end]
            .copy_from_slice(&source_texture.data[texture_begin..texture_end]);
    }
}

/*----------------------------------------------------------------------------*/

/// Creates the Sprite that shows our assembled map.
///
/// This system gets Commands, which is a queue which can be used to spawn or
/// remove Elements from the World, which is basically the container for
/// everything in a Bevy game. (Where does this "World" come from?
/// We don't need to set it up; it is created as part of the App in main,
/// below.)
///
pub fn create_map_sprite_system(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mapengine_map: Res<crate::MapEngineMap>,
) {
    // The resource MapEngineMap should already be defined, including
    // a tiny empty texture.
    // This line does two things: adds that texture as a global resource,
    // and also gets us a handle to put into the SpriteBundle as a material.
    // Bevy needs both of these things in order to actually render.
    let map_texture_handle = textures.add(mapengine_map.texture.clone());

    // And here is our "sprite" which shows the whole map. I use "sprite"
    // in scare quotes because it might be quite a bit larger than what
    // that name normally implies, but, hey, we work with what we have.
    // We add the MapEngineSprite component so we can keep this straight
    // from any other sprites.
    commands
        .spawn(SpriteBundle {
            material: materials.add(map_texture_handle.into()),
            ..Default::default()
        })
        .with(crate::MapEngineSprite);
}

/// Draw spaces that need updated onto the map texture.
///
/// TODO Handle removal of spaces, not just addition
///
/// This runs every frame when the engine is in the Running state, so it
/// is important to not do slow things. Unfortunately, because Bevy
/// does not yet support GPU texture-to-texture copy or batched rendering,
/// there are more slow operations here than ideal.
///
/// The first Query here returns MapSpace entities that have MapSpaceRefreshNeeded
/// And the second one gets us all of our map sprites. (There might be
/// more than one for a different view into the same map; that's to be implemented.)
pub fn maptexture_update_system(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mapengine_map: ResMut<crate::MapEngineMap>,
    mapspaces: Query<(Entity, &crate::MapSpace), With<crate::MapSpaceRefreshNeeded>>,
    mapsprites: Query<&Handle<ColorMaterial>, With<crate::MapEngineSprite>>,
) {
    // MapSpaces are entities in the World. They should be tagged
    // with MapSpaceRefreshNeeded if they've changed in appearance,
    // which will cause this system to get them.

    // This first pass gathers information needed to size the map texture,
    // and if we need to do anything at all.
    // TODO This doubles the number of times we go through the list;
    // consider if it is really the best way. (One idea for an alternate
    // approach: check the map size when spawning a new mapspace, and
    // mark it to grow if need be then.)
    let mut count = 0;
    for (_entity, mapspace) in mapspaces.iter() {
        // Find the furthest-from 0,0 rows and columns.
        // The +1 is because we are zero-indexed, so if everything is in col 0
        // we still need a space_width-wide map.
        mapengine_map.cols = cmp::max(mapengine_map.cols, mapspace.col + 1);
        mapengine_map.rows = cmp::max(mapengine_map.rows, mapspace.row + 1);
        count += 1;
    }
    // If there aren't any, exit now.
    // TODO Refactor so this happens instantly at the beginning of the system
    if count == 0 {
        return;
    }

    // We need to copy these out of the resource because later there's
    // a mutable+immutable borrow attempt if we don't have our own copy.
    let space_width_pixels = mapengine_map.space_width_pixels;
    let space_height_pixels = mapengine_map.space_height_pixels;

    // If our existing texture is too small, create a new bigger one.
    if mapengine_map.texture.size.width < mapengine_map.cols as u32 * space_width_pixels as u32
        || mapengine_map.texture.size.height
            < mapengine_map.rows as u32 * space_height_pixels as u32
    {
        println!(
            "Resizing map texture from {:?}×{:?} to {:?}×{:?}.",
            mapengine_map.texture.size.width,
            mapengine_map.texture.size.height,
            mapengine_map.cols as u32 * space_width_pixels as u32,
            mapengine_map.rows as u32 * space_height_pixels as u32,
        );
        let mut new_texture = Texture::new_fill(
            Extent3d::new(
                mapengine_map.cols as u32 * space_width_pixels as u32,
                mapengine_map.rows as u32 * space_height_pixels as u32,
                1,
            ),
            TextureDimension::D2,
            // transparent
            // FUTURE make this configurable
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
        );

        // copy the old texture to the new one — 0,0 for top left
        copy_texture(&mut new_texture, &mapengine_map.texture, 0, 0);

        // and swap it in.
        mapengine_map.texture = new_texture;
    }

    // And now we iterate through again and do the actual copying
    for (entity, mapspace) in mapspaces.iter() {
        // Each space has a handle to the texture which should represent it visually
        match textures.get(&mapspace.texture_handle) {
            Some(space_texture) => {
                copy_texture(
                    &mut mapengine_map.texture,
                    &space_texture,
                    mapspace.col as usize * space_width_pixels,
                    mapspace.row as usize * space_height_pixels,
                );
            }
            None => {
                eprintln!("For some reason, a texture is missing.");
                std::process::exit(2);
            }
        };
        commands.remove_one::<crate::MapSpaceRefreshNeeded>(entity);
    }

    // As above, this does two things: gets us the handle to put into the
    // sprite, and also adds the texture as a global resource. Bevy
    // needs both of these things to happen in order to actually render.
    let map_texture_handle = textures.add(mapengine_map.texture.clone());

    // We only need to grab the first map sprite, because they
    // all share the same material. And if there isn't one, that's fine;
    // we'll update it once there is in a future pass.
    if let Some(material) = mapsprites.iter().next() {
        materials.get_mut(material).unwrap().texture = Some(map_texture_handle);
    };
}
