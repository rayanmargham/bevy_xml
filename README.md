# Bevy Sparrow XML Parser

This Crate is a plugin for [Bevy](https://bevyengine.org/) to parse SpriteSheet XMLs! Most Useful in using assets from Adobe Animate.

## Using this Crate

Here below is some example code on how'd you use the crate:

```rust ignore
use bevy::prelude::*;
use bevy_xml::*;

fn your_startup_system(mut texture_atlases: ResMut<Assets<TextureAtlas>>, asset_loader: Res<AssetServer>)
{
    let bfs = texture_atlases.add(TextureAtlas::new_empty(asset_loader.load("images/bf.png"), Vec2::new(8192.0, 4096.0))); // handle
    let Some(bf) = texture_atlases.get_mut(&bfs) else { return };
    let xml = SpriteXMLBundle::new("assets/images/bf.xml".to_string(), &bfs, bf);

    match xml {
        Some(c) =>
        {
            commands.spawn(c);
        },
        None =>
        {
            error!("Failed to Parse XML!");
        }

    }
}

// system
fn update_frame_system(mut query: Query<(&mut SpriteXML, &mut TextureAtlasSprite, &mut Transform), With<SpriteXML>>)
{
    for (mut xml, mut sprite, mut trans) in query.iter_mut()
    {
        xml.get_next_frame(&mut sprite, &mut trans);
    }
}
```

## What this Crate gives you

This crate only gives you new() and get_next_frame(), offsets are by the crate. The animation system is for you to implement!

## Disclaimer!

I've never published a crate before so if there are any issues please give me feedback!