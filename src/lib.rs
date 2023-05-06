use std::time::Duration;

use bevy::prelude::*;
use bevy_sprite3d::{AtlasSprite3d, AtlasSprite3dBundle, AtlasSprite3dComponent, Sprite3dParams};
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

pub struct XAnimation {
    name: String,
    idx: usize,
    idx_range: Vec<usize>,
    looped: bool,
    should_go: bool,
    timer: Timer,
    finished: bool,
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

#[derive(Component)]
pub struct SpriteXML {
    pub offsets: Offsets,
    pub xml: XML,
    pub animations: Vec<XAnimation>,
    pub cur_anim: Option<usize>,
}
#[derive(Component)]
pub struct SpriteXML3D {
    pub offsets: Offsets,
    pub xml: XML,
    pub animations: Vec<XAnimation>,
    pub cur_anim: Option<usize>,
}

#[derive(Bundle)]
pub struct SpriteXMLBundle {
    pub sprite_sheet: SpriteSheetBundle,
    pub spritexml: SpriteXML,
}
#[derive(Bundle)]
pub struct SpriteXMLBundle3D {
    pub sprite_sheet: AtlasSprite3dBundle,
    pub spritexml: SpriteXML3D,
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
                        let sheet_bundle = SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(0),
                            texture_atlas: atlas_handle.clone(),
                            ..default()
                        };
                        let mut temp_offsets: Offsets = Offsets {
                            offsets_vec: Vec::new(),
                        };

                        for texture in xml.subtexture.iter() {
                            let name = &texture.name;

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
                                xml,
                                animations: Vec::new(),
                                cur_anim: None,
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
            translation.translation.z,
        ) * translation.scale;

        // add new offset :cool:
        let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
        translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
    }
    pub fn add_anim_from_prefix(
        &mut self,
        animationtoadd: String,
        looped: bool,
        fps: u16,
    ) -> Result<(), &str> {
        let fps_in_ms = 1.0 / fps as f64;
        let mut idx_rng = Vec::new();
        for subtex in self.xml.subtexture.iter() {
            if !subtex.name.starts_with(&animationtoadd) {
                continue;
            }
            idx_rng.push(
                self.xml
                    .subtexture
                    .iter()
                    .position(|z| z == subtex)
                    .unwrap(),
            );
        }
        if idx_rng.len() >= 1 {
            self.animations.push(XAnimation {
                name: animationtoadd,
                idx_range: idx_rng,
                idx: 0,
                should_go: false,
                looped,
                timer: Timer::new(Duration::from_secs_f64(fps_in_ms), TimerMode::Repeating),
                finished: false,
            });
            Ok(())
        } else {
            Err("Failed to find Animation, Does it exist?")
        }
    }
    pub fn get_next_frame_of_anim(
        &mut self,
        sprite: &mut TextureAtlasSprite,
        translation: &mut Transform,
        anim_idx: usize,
    ) {
        let mut animation = &mut self.animations[anim_idx];

        if animation.idx >= animation.idx_range.len() - 1 {
            // out of bounds handle looped or whatever
            if animation.looped {
                let last_index: usize = sprite.index;
                animation.idx = 0;
                sprite.index = animation.idx_range[animation.idx];

                // reapply offsets
                translation.translation -= Vec3::new(
                    self.offsets.offsets_vec[last_index].x,
                    self.offsets.offsets_vec[last_index].y,
                    0.0,
                ) * translation.scale;
                let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
                translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
            } else {
                animation.finished = true;
                return;
            }
        } else {
            let last_index: usize = sprite.index;
            if animation.should_go == true {
                animation.idx += 1;
            } else {
                animation.should_go = true;
            }

            sprite.index = animation.idx_range[animation.idx];

            // reapply offsets (assumes you already applied offsets using apply_offsets() for the first frame if you just spawned the sprite)
            translation.translation -= Vec3::new(
                self.offsets.offsets_vec[last_index].x,
                self.offsets.offsets_vec[last_index].y,
                0.0,
            ) * translation.scale;
            let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
            translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
        }
    }

    pub fn apply_offsets(&self, sprite: &TextureAtlasSprite, translation: &mut Transform) {
        let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
        translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
    }

    pub fn reset_anim_idx(&mut self) -> Result<(), &str> {
        match self.cur_anim {
            Some(idx) => {
                self.animations[idx].idx = 0;
                Ok(())
            }
            None => Err("There's no Current Animation!"),
        }
    }
    pub fn set_anim(
        &mut self,
        animationtoset: String,
        sprite: &mut TextureAtlasSprite,
        startatzero: bool,
    ) -> Result<(), &str> {
        let mut anim_idx: usize = 9999;

        for i in self.animations.iter() {
            if !i.name.starts_with(&animationtoset) {
                continue;
            }

            anim_idx = self
                .animations
                .iter()
                .position(|z| z.name == i.name)
                .unwrap();
            break;
        }
        if anim_idx != 9999 {
            self.cur_anim = Some(anim_idx);
            if startatzero {
                sprite.index = self.animations[self.cur_anim.unwrap()].idx_range[0];
            }

            Ok(())
        } else {
            Err("Couldn't Find Animation, Did you add it?")
        }
    }
}

pub fn tick_animations(
    mut query: Query<(&mut SpriteXML, &mut Transform, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (mut xml, mut translation, mut sprite) in query.iter_mut() {
        match xml.cur_anim {
            Some(idx) => {
                let animation = &mut xml.animations[idx];
                animation.timer.tick(time.delta());

                if animation.timer.just_finished() {
                    xml.get_next_frame_of_anim(&mut sprite, &mut translation, idx);
                }
            }
            None => {}
        }
    }
}

impl SpriteXMLBundle3D {
    pub fn new(
        xml_path: String,
        atlas_handle: &Handle<TextureAtlas>,
        params: &mut Sprite3dParams,
        unlit: bool,
    ) -> Option<Self> {
        let file = std::fs::read_to_string(xml_path);

        match file {
            Ok(f) => {
                let x = f.replace("\u{feff}", "");
                let xml: Result<XML, serde_xml_rs::Error> = from_str(&x);
                match xml {
                    Ok(xml) => {
                        let sheet_bundle = AtlasSprite3d {
                            atlas: atlas_handle.clone(),
                            unlit: unlit,
                            ..default()
                        };
                        let mut temp_offsets: Offsets = Offsets {
                            offsets_vec: Vec::new(),
                        };
                        let Some(hd) = params.atlases.get_mut(&atlas_handle) else { return None};
                        for texture in xml.subtexture.iter() {
                            let name = &texture.name;

                            let rect = Rect::new(
                                texture.x as f32,
                                texture.y as f32,
                                texture.x as f32 + texture.width as f32,
                                texture.y as f32 + texture.height as f32,
                            );

                            temp_offsets.offsets_vec.push(Vec2::new(
                                -(texture.frameX as f32 / 2f32) / 100f32,
                                (texture.frameY as f32 / 2f32) / 100f32,
                            ));

                            hd.add_texture(rect);
                        }
                        // All done!
                        return Some(SpriteXMLBundle3D {
                            sprite_sheet: sheet_bundle.bundle(params),
                            spritexml: SpriteXML3D {
                                offsets: temp_offsets,
                                xml,
                                animations: Vec::new(),
                                cur_anim: None,
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

impl SpriteXML3D {
    pub fn add_anim_from_prefix(
        &mut self,
        animationtoadd: String,
        looped: bool,
        fps: u16,
    ) -> Result<(), &str> {
        let fps_in_ms = 1.0 / fps as f64;
        let mut idx_rng = Vec::new();
        for subtex in self.xml.subtexture.iter() {
            if !subtex.name.starts_with(&animationtoadd) {
                continue;
            }
            idx_rng.push(
                self.xml
                    .subtexture
                    .iter()
                    .position(|z| z == subtex)
                    .unwrap(),
            );
        }
        if idx_rng.len() >= 1 {
            self.animations.push(XAnimation {
                name: animationtoadd,
                idx_range: idx_rng,
                idx: 0,
                should_go: false,
                looped,
                timer: Timer::new(Duration::from_secs_f64(fps_in_ms), TimerMode::Repeating),
                finished: false,
            });
            Ok(())
        } else {
            Err("Failed to find Animation, Does it exist?")
        }
    }
    pub fn get_next_frame_of_anim(
        &mut self,
        sprite: &mut AtlasSprite3dComponent,
        translation: &mut Transform,
        anim_idx: usize,
    ) {
        let mut animation = &mut self.animations[anim_idx];

        if animation.idx >= animation.idx_range.len() - 1 {
            // out of bounds handle looped or whatever
            if animation.looped {
                let last_index: usize = sprite.index;
                animation.idx = 0;
                sprite.index = animation.idx_range[animation.idx];

                // reapply offsets
                translation.translation -= Vec3::new(
                    self.offsets.offsets_vec[last_index].x,
                    self.offsets.offsets_vec[last_index].y,
                    0.0,
                ) * translation.scale;
                let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
                translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
            } else {
                animation.finished = true;
                return;
            }
        } else {
            let last_index: usize = sprite.index;
            if animation.should_go == true {
                animation.idx += 1;
            } else {
                animation.should_go = true;
            }

            sprite.index = animation.idx_range[animation.idx];
            // reapply offsets (assumes you already applied offsets using apply_offsets() for the first frame if you just spawned the sprite)
            translation.translation -= Vec3::new(
                self.offsets.offsets_vec[last_index].x,
                self.offsets.offsets_vec[last_index].y,
                0.0,
            ) * translation.scale;
            let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
            translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
        }
    }

    pub fn apply_offsets(&self, sprite: &AtlasSprite3dComponent, translation: &mut Transform) {
        let offset: Vec2 = self.offsets.offsets_vec[sprite.index];
        translation.translation += Vec3::new(offset.x, offset.y, 0.0) * translation.scale;
    }

    pub fn reset_anim_idx(&mut self) -> Result<(), &str> {
        match self.cur_anim {
            Some(idx) => {
                self.animations[idx].idx = 0;
                Ok(())
            }
            None => Err("There's no Current Animation!"),
        }
    }
    pub fn set_anim(
        &mut self,
        animationtoset: String,
        sprite: &mut AtlasSprite3dComponent,
        startatzero: bool,
    ) -> Result<(), &str> {
        let mut anim_idx: usize = 9999;

        for i in self.animations.iter() {
            if !i.name.starts_with(&animationtoset) {
                continue;
            }

            anim_idx = self
                .animations
                .iter()
                .position(|z| z.name == i.name)
                .unwrap();
            break;
        }
        if anim_idx != 9999 {
            self.cur_anim = Some(anim_idx);
            if startatzero {
                sprite.index = self.animations[self.cur_anim.unwrap()].idx_range[0];
            }

            Ok(())
        } else {
            Err("Couldn't Find Animation, Did you add it?")
        }
    }
}

pub fn tick_animations_3D(
    mut query: Query<(
        &mut SpriteXML3D,
        &mut Transform,
        &mut AtlasSprite3dComponent,
    )>,
    time: Res<Time>,
) {
    for (mut xml, mut translation, mut sprite) in query.iter_mut() {
        match xml.cur_anim {
            Some(idx) => {
                let animation = &mut xml.animations[idx];
                animation.timer.tick(time.delta());

                if animation.timer.just_finished() {
                    xml.get_next_frame_of_anim(&mut sprite, &mut translation, idx);
                }
            }
            None => {}
        }
    }
}
