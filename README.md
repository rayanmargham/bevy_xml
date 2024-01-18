[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking)

# Bevy Sparrow XML Parser

This Crate is a plugin for [Bevy](https://bevyengine.org/) to parse SpriteSheet XMLs! Most Useful in using assets from Adobe Animate.

## Using this Crate

Here below is some example code on how'd you use the crate:

```rust ignore
use bevy::prelude::*;
use bevy_xml::*;

fn main()
{
    app
        .add_system(bevy_xml::tick_animations)
        .run();
}
fn your_startup_system(mut texture_atlases: ResMut<Assets<TextureAtlas>>, asset_loader: Res<AssetServer>)
{
    let bfs = texture_atlases.add(TextureAtlas::new_empty(asset_loader.load("images/bf.png"), Vec2::new(8192.0, 4096.0))); // handle
    let Some(bf) = texture_atlases.get_mut(&bfs) else { return };
    let xml = SpriteXMLBundle::new("assets/images/bf.xml".to_string(), &bfs, bf);

    match xml {
        Some(c) =>
        {
            c.add_anim_from_prefix("Epic Animation", false, 24); // anim name, is the animation looped?, fps
            c.apply_offsets(); // applies inital offset
            c.set_anim("Epic Animation", bf, true); // sets the current animation
                    // anim name^  sprite^   ^ set the current frame to zero?
            commands.spawn(c);
        },
        None =>
        {
            error!("Failed to Parse XML!");
        }

    }
}


```

## What this Crate gives you

This crate now has an animation system, support for 3D sprites and much more!

## Disclaimer!

I've never published a crate before so if there are any issues please give me feedback!
