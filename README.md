bevy_mapengine
==============

A 2D tilemap plugin for the Bevy game engine.

Screenshot
----------

![This is all that 0.0.4 does.](examples/screenshots/screenshot-0.0.4.png)

What is this?
-------------

This is an experimental plugin for the [Bevy](https://bevyengine.org/)
game engine.

It is intended to make it easy and quick to make top-down or side-view
games in Bevy.

Try it!
-------

Run the demo with:

    cargo run --example demo

The tiles used in this demo come from a “No Rights Reserved”
[CC0](https://creativecommons.org/share-your-work/public-domain/cc0/)
art pack from [Kenney](https://kenney.nl/assets/medieval-rts).

The demo is well-commented and currently serves as usage documentation.

Bevy version?
-------------

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

I'm going to try to track the latest stable Bevy release. While Bevy is
in rapid development, I'm not going to attempt anything other than
updating to the new version whenever one appears.

Important Performance Notes
---------------------------

Bevy doesn't currently implement a way to draw textures on other
textures. Hopefully this will be implemented soon. (See
[this issue](https://github.com/bevyengine/bevy/issues/1207#issuecomment-800602680)
for details.) In the meantime, this plugin copies image pixel data using
the CPU in a pretty simple way. This is not good for anything but
infrequent updates. Additionally, there is no render batching, so
updating a bunch of map cells at once will be quite slow.

However, this will do while working on basic functionality, and may even
be useful for simple games. Luckily, all of this is behind the scenes.
When Bevy gets support for doing this in a fast way, switching to that
should be seamless the point of view of a user of the plugin.

First priorities
----------------

1. Load and display a grid of cells.
2. Example which shows mouse-over
3. Scrolling (with WASD and mouse examples)
4. Bounds checking when scrolling
5. Zoom (with ZXC and mouse examples)

Medium-term
-----------

* Cope with resizeable windows
* Performance: don't render offscreen (but do on zoom or scroll!)
* Layers
  - to be decided: separate entities or multiple textures in same cell?
* Position info for non-mapped sprites.
* Different views into same map (for mini-map)
* Swap texture sizes based on zoom

After that...
-------------

* (Optional) Automatic selection of border tile images for prettiness
* Pathfinding?
  - ideally integration rather than diy
* Collision detection
* Arbitrary rotation?
* Chunks for arbitrarily-large maps
  - loaded from disk or generated on the fly

Not currently considering...
----------------------------

I like these things but am not focusing on:

* isometric grids
* hex grids

I would be open to PRs which implement them, however.

What about bevy_tilemap?
------------------------

[Bevy Tilemap](https://bevyengine.org/) is another tilemap
implementation, with some similar goals. It's cool too. Both projects
intend to make it fast to get up and going with minimal fiddling. We're
kind of coming at the project from different directions; Bevy Tilemap is
initially focused on chunk loading and related things, while I'm focused
more on the UI and things like correlating clicks to tiles.

With Bevy Tilemap, tiles are added to a data structure held by the
Tilemap itself. Here instead each cell is actually an Entity in Bevy.

Who are you then?
-----------------

Not a Bevy expert, nor a Rust one. Just wanted to make something for
myself, and hopefully useful to others as well.

License
-------

This is free and open source software under the MIT license.