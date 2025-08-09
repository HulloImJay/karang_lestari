use crate::object_manager;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Handle};
use bevy::gltf::GltfAssetLabel;
use bevy::image::Image;
use bevy::prelude::{Res, ResMut, Resource, Scene};
use glam::Vec2;
use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};

#[derive(Default)]
pub struct ObjectManagerPlugin;

impl Plugin for ObjectManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ObjectManager::new(vec![
            ObjectDefinition {
                name: "acropora_2_anten_v".into(),
                path: "models/objects/acropora_2_anten_v.glb".into(),
                orientation_type: OrientationType::VerticalForward,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_3_anten".into(),
                path: "models/objects/acropora_3_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_4_anten".into(),
                path: "models/objects/acropora_4_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_abrolhosensis_anten".into(),
                path: "models/objects/acropora_abrolhosensis_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_anten".into(),
                path: "models/objects/acropora_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_cytherea_2_komang".into(),
                path: "models/objects/acropora_cytherea_2_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_cytherea_anten".into(),
                path: "models/objects/acropora_cytherea_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_cytherea_pink_2_komang".into(),
                path: "models/objects/acropora_cytherea_pink_2_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_cytherea_pink_anten".into(),
                path: "models/objects/acropora_cytherea_pink_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_natalensis_komang".into(),
                path: "models/objects/acropora_natalensis_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "acropora_sukarnoi_komang".into(),
                path: "models/objects/acropora_sukarnoi_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "big_brain_paula_q".into(),
                path: "models/objects/big_brain_paula_q.glb".into(),
                orientation_type: OrientationType::Quarter,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "big_sponge_paula".into(),
                path: "models/objects/big_sponge_paula.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(2.0, 2.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "block_arch_paula".into(),
                path: "models/objects/block_arch_paula.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(2.0, 2.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "concrete_dome_anten".into(),
                path: "models/objects/concrete_dome_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "concrete_turtle_anten".into(),
                path: "models/objects/concrete_turtle_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "concrete_turtle_komang".into(),
                path: "models/objects/concrete_turtle_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "coral_table_komang".into(),
                path: "models/objects/coral_table_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 1.0..1.0,
            },
            ObjectDefinition {
                name: "diploastrea_heliopora_komang".into(),
                path: "models/objects/diploastrea_heliopora_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "favia_komang".into(),
                path: "models/objects/favia_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "leptoria_phrygia_2_anten".into(),
                path: "models/objects/leptoria_phrygia_2_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "leptoria_phrygia_komang".into(),
                path: "models/objects/leptoria_phrygia_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "les_komang".into(),
                path: "models/objects/les_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "lobophytum_komang".into(),
                path: "models/objects/lobophytum_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "mixed_coral_anten".into(),
                path: "models/objects/mixed_coral_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "montipora_digitata_komang".into(),
                path: "models/objects/montipora_digitata_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "pincushion_starfish_komang".into(),
                path: "models/objects/pincushion_starfish_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "pocillopora_meandrina_komang".into(),
                path: "models/objects/pocillopora_meandrina_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_2_anten".into(),
                path: "models/objects/porites_lutea_2_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_3_anten".into(),
                path: "models/objects/porites_lutea_3_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_4_komang".into(),
                path: "models/objects/porites_lutea_4_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_5_anten".into(),
                path: "models/objects/porites_lutea_5_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_6_anten".into(),
                path: "models/objects/porites_lutea_6_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_7_anten".into(),
                path: "models/objects/porites_lutea_7_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "porites_lutea_anten_q".into(),
                path: "models/objects/porites_lutea_anten_q.glb".into(),
                orientation_type: OrientationType::Quarter,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "small_yellow_coral_paula".into(),
                path: "models/objects/small_yellow_coral_paula.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "sponge_4_anten".into(),
                path: "models/objects/sponge_4_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "sponge_5_anten".into(),
                path: "models/objects/sponge_5_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "sponge_komang".into(),
                path: "models/objects/sponge_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "starfish_2_komang".into(),
                path: "models/objects/starfish_2_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "starfish_komang".into(),
                path: "models/objects/starfish_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "tendrils".into(),
                path: "models/objects/tendrils.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.3, 0.3),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "tunnel_komang".into(),
                path: "models/objects/tunnel_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "turbinaria_2_anten".into(),
                path: "models/objects/turbinaria_2_anten.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.2, 0.2),
                scale: 0.66..1.5,
            },
            ObjectDefinition {
                name: "turbinaria_photos_komang".into(),
                path: "models/objects/turbinaria_photos_komang.glb".into(),
                orientation_type: OrientationType::HorizontalFree,
                size: Vec2::new(0.5, 1.0),
                scale: 0.66..1.5,
            },
        ]))
        .add_systems(Startup, object_manager::asset_manager_init);
    }
}

#[derive(Clone, Debug)]
pub enum OrientationType {
    HorizontalFree,
    VerticalForward,
    Quarter,
}

#[derive(Clone, Debug)]
pub struct ObjectDefinition {
    pub name: String,
    pub path: String,
    pub orientation_type: OrientationType,
    pub size: Vec2,
    pub scale: Range<f32>,
}

#[derive(Clone, Debug)]
pub struct ObjectData {
    pub object_definition: ObjectDefinition,
    pub model_handle: Handle<Scene>,
}

#[derive(Resource)]
pub struct ObjectManager {
    pub objects: HashMap<String, ObjectData>,
    pub floor_texture: Handle<Image>,
}

impl ObjectManager {
    pub fn new(object_definitions: Vec<ObjectDefinition>) -> Self {
        let mut objects_data = HashMap::new();
        for def in object_definitions.iter() {
            objects_data.insert(
                def.name.clone(),
                ObjectData {
                    object_definition: def.clone(),
                    model_handle: Default::default(),
                },
            );
        }
        ObjectManager {
            objects: objects_data,
            floor_texture: Handle::default(),
        }
    }


    /// Grab a reference to an object (with its handle if loaded)
    pub fn get(&self, name: &str) -> Option<&ObjectData> {
        self.objects.get(name)
    }
}

pub fn asset_manager_init(
    asset_server: Res<AssetServer>,
    mut object_manager: ResMut<ObjectManager>,
) {
    for obj in object_manager.objects.values_mut() {
        let handle = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(obj.object_definition.path.clone()));
        obj.model_handle = handle;
    }

    object_manager.floor_texture = asset_server.load("textures/temp_floor_2.png");
}

pub fn asset_manager_ready(
    object_manger: Res<ObjectManager>,
    asset_server: Res<AssetServer>,
) -> bool {
    for obj in object_manger.objects.values() {
        if !asset_server.is_loaded(&obj.model_handle) {
            return false;
        }
    }

    true
}
