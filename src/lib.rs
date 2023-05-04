#![allow(non_snake_case)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

// XML Format Serialization

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SubTexture {
    pub name: String,
    pub x: u32,
    pub y: u32,

    pub height: u32,
    pub width: u32,

    #[serde(default = "default_i32")]
    pub frameX: i32,
    #[serde(default = "default_i32")]
    pub frameY: i32,

    // used instead of height and width for hitbox calculations and origin calculations
    #[serde(default = "default_u32")]
    pub frameWidth: u32,
    #[serde(default = "default_u32")]
    pub frameHeight: u32,
}

fn default_i32() -> i32 {
    0
}
fn default_u32() -> u32 {
    0
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct XML {
    #[serde(rename = "SubTexture")]
    subtexture: Vec<SubTexture>,
}

#[derive(Clone, Debug)]
pub struct Offsets {
    pub offsets_vec: Vec<Vec2>,
}

#[derive(Component, Clone)]
pub struct SpriteXML {
    pub offsets: Offsets,
}

#[derive(Bundle, Clone)]
pub struct SpriteXMLBundle {
    pub sprite_sheet: SpriteSheetBundle,
    pub spritexml: SpriteXML,
}

impl SpriteXMLBundle {
    pub fn new(
        xml_path: String,
        atlas_handle: &Handle<TextureAtlas>,
        modify: &mut TextureAtlas,
    ) -> Option<Self> {
        let file = std::fs::read_to_string(xml_path);

        match file {
            Ok(f) => {
                let x = f.replace("\u{feff}", "");
                let xml: Result<XML, serde_xml_rs::Error> = from_str(&x);
                match xml {
                    Ok(xml) => {
                        let mut sheet_bundle = SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(0),
                            texture_atlas: atlas_handle.clone(),
                            ..default()
                        };
                        let mut temp_offsets: Offsets = Offsets {
                            offsets_vec: Vec::new(),
                        };

                        for texture in xml.subtexture.iter() {
                            let name = &texture.name;

                            // if !name.starts_with("BF idle dance") {
                            //     continue;
                            // }

                            let rect = Rect::new(
                                texture.x as f32,
                                texture.y as f32,
                                texture.x as f32 + texture.width as f32,
                                texture.y as f32 + texture.height as f32,
                            );

                            temp_offsets.offsets_vec.push(Vec2::new(
                                -texture.frameX as f32 / 2f32,
                                texture.frameY as f32 / 2f32,
                            ));

                            modify.add_texture(rect);
                        }
                        // All done!
                        return Some(SpriteXMLBundle {
                            sprite_sheet: sheet_bundle,
                            spritexml: SpriteXML {
                                offsets: temp_offsets,
                            },
                        });
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                return None;
            }
        }
        return None;
    }
}

impl SpriteXML {
    pub fn get_next_frame(&mut self, sprite: &mut TextureAtlasSprite, translation: &mut Transform) {
        // this makes more readable code i guess!!!
        let total_frames: usize = self.offsets.offsets_vec.len();
        let last_index: usize = sprite.index;
        sprite.index += 1;

        // no dumb overflow errors!!! + looping ig
        if last_index >= total_frames - 1 {
            sprite.index = 0;
        }
        
        // remove old offset lmao!!!
        translation.translation -= Vec3::new(
            self.offsets.offsets_vec[last_index].x,
            self.offsets.offsets_vec[last_index].y,
            0.0,
        ) * translation.scale;

        // add new offset :cool:
        let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
        translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
    }
}
