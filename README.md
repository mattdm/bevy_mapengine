bevy_mapengine
==============

A 2D tilemap plugin for the Bevy game engine.

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

Bevy version?
-------------

I'm going to try to track the latest stable Bevy release. While Bevy is
in rapid development, I'm not going to attempt anything other than
updating to the new version whenever one appears.

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

License
-------

This is free and open source software under the MIT license.