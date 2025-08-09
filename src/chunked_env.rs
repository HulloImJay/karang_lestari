use crate::object_manager;
use crate::object_manager::ObjectManager;
use crate::region_sampler::RegionSampler;
use bevy::asset::RenderAssetUsages;
use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

/// Call `.add_plugin(ChunkedEnvironmentPlugin::default())` in your App.
pub struct ChunkedEnvironmentPlugin;

impl Default for ChunkedEnvironmentPlugin {
    fn default() -> Self {
        ChunkedEnvironmentPlugin
    }
}

impl Plugin for ChunkedEnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app
            // how many chunks along ±X and ±Z, and how large each one is
            .insert_resource(ChunkSettings {
                radius: 10,
                chunk_size: 2.0,
            })
            // tracks loaded chunk entities
            .init_resource::<ChunkManager>()
            // runs every frame after camera has moved
            .add_systems(
                Update,
                chunk_manager_system.run_if(object_manager::asset_manager_ready),
            );
    }
}

/// Configure how many chunks to keep loaded, and their size
#[derive(Resource)]
pub struct ChunkSettings {
    pub radius: i32,
    pub chunk_size: f32,
}

/// Keeps a map from chunk‐coords → spawned Entity
#[derive(Resource)]
pub struct ChunkManager {
    loaded: HashMap<IVec2, Entity>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        ChunkManager {
            loaded: HashMap::new(),
        }
    }
}

/// Queries the camera each frame, figures out which chunk‐coords
/// should be present, spawns new ones via `spawn_chunk`, and
/// despawns the ones that fall out of range.
fn chunk_manager_system(
    settings: Res<ChunkSettings>,
    mut manager: ResMut<ChunkManager>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cam_tf: Query<&GlobalTransform, With<Camera3d>>,
    coral_assets: Res<ObjectManager>,
    asset_server: Res<AssetServer>,
    region_sampler: Res<RegionSampler>,
) {
    let cam_pos = cam_tf.single().unwrap().translation();
    let cs = settings.chunk_size;
    // determine which chunk the camera is in
    let cam_chunk = IVec2::new(
        (cam_pos.x / cs).floor() as i32,
        (cam_pos.z / cs).floor() as i32,
    );

    // build the set of coords we *want*
    let mut wanted = HashSet::new();
    for dx in -settings.radius..=settings.radius {
        for dz in -settings.radius..=settings.radius {
            wanted.insert(IVec2::new(cam_chunk.x + dx, cam_chunk.y + dz));
        }
    }

    // spawn any missing chunks
    for &coord in wanted.iter() {
        if !manager.loaded.contains_key(&coord) {
            let ent = spawn_chunk(
                &mut commands,
                &mut meshes,
                &mut materials,
                coord,
                settings.chunk_size,
                &coral_assets,
                &asset_server,
                &region_sampler,
            );
            manager.loaded.insert(coord, ent);
        }
    }

    // despawn out-of-range chunks
    manager.loaded.retain(|&coord, &mut ent| {
        if wanted.contains(&coord) {
            true
        } else {
            commands.entity(ent).despawn();
            false
        }
    });
}

/// Example chunk‐factory: one plane + four little cubes.
/// Edit this to plug in your own procedural geometry!
fn spawn_chunk(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    coord: IVec2,
    chunk_size: f32,
    object_manager: &Res<ObjectManager>,
    asset_server: &Res<AssetServer>,
    region_sampler: &Res<RegionSampler>,
) -> Entity {
    let mut rng = rand::rng();
    let half = chunk_size * 0.5;
    // center position of this chunk in world coords
    let world_x = coord.x as f32 * chunk_size + half;
    let world_z = coord.y as f32 * chunk_size + half;

    // prepare shared mesh/material handles
    // let plane = meshes.add(Mesh::from(shape::Plane { size: chunk_size }));
    // let plane_mat = materials.add(Color::from(Hsla { 0: Default::default() }).into());

    // let plane = Mesh3d(
    //     meshes.add(
    //         Plane3d::default()
    //             .mesh()
    //             .size(chunk_size, chunk_size)
    //             .subdivisions(10),
    //     ),
    // );
    // let plane_mat = MeshMaterial3d(materials.add(Color::from(SILVER)));
    let plane_mat = MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(object_manager.floor_texture.clone()),
        alpha_mode: AlphaMode::Opaque,
        unlit: false,
        perceptual_roughness: 0.95,
        metallic: 0.6,
        ..default()
    }));
    let plane = create_heightmap_mesh3d(
        &mut meshes,
        &region_sampler,
        chunk_size,
        10,
        Vec2::new(world_x, world_z),
    );

    // spawn a parent so we can despawn the whole chunk at once
    let parent = commands
        .spawn((
            Name::new(format!("Chunk({},{})", coord.x, coord.y)),
            Transform::from_translation(Vec3::new(world_x, 0.0, world_z)),
            Visibility::default(),
        ))
        .id();

    // now attach the floor + some cubes
    commands.entity(parent).with_children(|parent| {
        // floor
        parent.spawn((plane, plane_mat));
        // four floating cubes at the corners
        let cube_size = half * 0.01;
        let cube_mesh = meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(cube_size, cube_size, cube_size) * 0.5,
        }));
        let cube_mat = materials.add(Color::srgb(0.8, 0.8, 0.9));
        let r_y_amt = 1.0;
        let r_xz_amt = chunk_size * 0.5;
        let r_s_amt = 1.0;
        let offsets = [
            Vec3::new(
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                2.5 + rng.random_range(-r_y_amt..r_y_amt),
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                2.5 + rng.random_range(-r_y_amt..r_y_amt),
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                2.5 + rng.random_range(-r_y_amt..r_y_amt),
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                2.5 + rng.random_range(-r_y_amt..r_y_amt),
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                5.0 + rng.random_range(-r_y_amt..r_y_amt),
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                5.0 + rng.random_range(-r_y_amt..r_y_amt),
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                5.0 + rng.random_range(-r_y_amt..r_y_amt),
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
            Vec3::new(
                -half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
                5.0 + rng.random_range(-r_y_amt..r_y_amt),
                half * 0.5 + rng.random_range(-r_xz_amt..r_xz_amt),
            ),
        ];
        for &off in &offsets {
            parent.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(cube_mat.clone()),
                Transform::from_translation(off),
            ));
        }

        let (regions, _) = region_sampler.sample_region(Vec2::new(world_x, world_z));
        let region = &region_sampler.regions[regions[0]];
        // let set_ind = rng.random_range(0..region.objects.len());
        // let obj_selection = region.objects[set_ind].clone();
        // let scene = &object_manager.get(&*obj_selection.name).unwrap().model_handle;

        let obj_name = region.pick_object();
        let obj = object_manager.get(&*obj_name).unwrap();
        let scene = &obj.model_handle;

        let mut local_pos = Vec3::new(
            rng.random_range(-r_xz_amt..r_s_amt),
            0.0,
            rng.random_range(-r_xz_amt..r_s_amt),
        );
        let world_pos_2d = Vec2::new(local_pos.x + world_x, local_pos.z + world_z);

        // local_pos.y = region_sampler.sample_surface_height(world_pos_2d) as f32;

        let size = obj.object_definition.size;
        let yaw = rng.random_range(-PI..PI);
        let yaw_matrix = Mat2::from_angle(yaw);

        let height = region_sampler.sample_surface_height(world_pos_2d) as f32;

        let sample = |offset: Vec2| region_sampler.sample_surface_height(world_pos_2d + offset) as f32;

        let v_right_2d = yaw_matrix * (Vec2::X * size.x);
        let v_up_2d = yaw_matrix * (Vec2::Y * size.y);

        let hl = sample(-v_right_2d);
        let hr = sample(v_right_2d);
        let hd = sample(-v_up_2d);
        let hu = sample(v_up_2d);

        // approximate partial derivatives
        let dh_dx = (hr - hl) / (2.0 * size.x);
        let dh_dz = (hu - hd) / (2.0 * size.y);

        // build the normal vector and normalize
        let normal = Vec3::new(-dh_dx, 1.0, -dh_dz).normalize();

        // rotate Y up → this normal
        let rot = Quat::from_rotation_arc(Vec3::Y, normal)
            * Quat::from_axis_angle(Vec3::Y, yaw);


        // **** SIMPLE ORIENTATION ****
        // let  ( rot_a, normal) = region_sampler.sample_surface_orientation(world_pos_2d, 0.1);
        //
        if normal.dot(Vec3::Y) < 0.88 {
            // println!("Discard!");
            return;
        }
        // let rot = rot_a * Quat::from_axis_angle(Vec3::Y, yaw);



        // **** SINK PHASE ****

        // Get the corners in 3D based on our actual orientation.
        let v_right_3d = rot * (Vec3::X * size.x);
        let v_up_3d = rot * (Vec3::Z * size.y);

        // Convert back to 2D for sampling the EXACT height over these points.
        // Optional: more accurate
        // let v_right_2d = Vec2::new(v_right_3d.x, v_right_3d.z);
        // let v_up_2d = Vec2::new(v_up_3d.x, v_up_3d.z);
        // let hl = sample(-v_right_2d);
        // let hr = sample(v_right_2d);
        // let hd = sample(-v_up_2d);
        // let hu = sample(v_up_2d);

        // Find the distance offsets.
        let d_l =  hl +v_right_3d.y;
        let d_r =  hr - v_right_3d.y;
        let d_d = hd + v_up_3d.y;
        let d_u = hu-v_up_3d.y;

        let y_pos_min = height.min(d_l).min(d_r).min(d_d).min(d_u);

        // println!("height = {}, y_pos_min = {}", height, y_pos_min);
        local_pos.y = y_pos_min;
        // local_pos.y = height;

        let scale = Vec3::ONE;// * rng.random_range(0.66..1.33);
        parent.spawn((
            SceneRoot(scene.clone()),
            Transform {
                translation: local_pos,
                rotation: rot,
                scale,
                ..default()
            },
        ));
    });

    parent
}

/// Compute a surface normal at x/z by sampling heights at ±eps,
/// then return the quaternion that rotates Y-up onto that normal.

/// Generates a heightmap Mesh from Perlin noise
pub fn generate_heightmap_mesh(
    region_sampler: &Res<RegionSampler>,
    chunk_size: f32,
    subdivisions: usize,
    world_offset: Vec2,
) -> Mesh {
    // const HEIGHT_SCALE: f32 = 1.0;
    // const SAMPLE_SCALE: f32 = 0.1;
    let verts_x = subdivisions + 1;
    let verts_z = subdivisions + 1;

    let mut positions = Vec::with_capacity(verts_x * verts_z);
    let mut normals = Vec::with_capacity(verts_x * verts_z);
    let mut uvs = Vec::with_capacity(verts_x * verts_z);

    for iz in 0..verts_z {
        for ix in 0..verts_x {
            let u = ix as f32 / subdivisions as f32;
            let v = iz as f32 / subdivisions as f32;

            let x_local = (u - 0.5) * chunk_size;
            let z_local = (v - 0.5) * chunk_size;
            // let x_sample = (x_local + world_offset.x) * SAMPLE_SCALE;
            // let z_sample = (z_local + world_offset.y) * SAMPLE_SCALE;
            //
            // let y = perlin.get([x_sample as f64, z_sample as f64]) as f32 * HEIGHT_SCALE;
            let y = region_sampler.sample_surface_height(Vec2::new(
                (x_local + world_offset.x),
                (z_local + world_offset.y),
            )) as f32;

            positions.push([x_local, y, z_local]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, v]);
        }
    }

    let mut indices = Vec::with_capacity(subdivisions * subdivisions * 6);
    for iz in 0..subdivisions {
        for ix in 0..subdivisions {
            let i0 = (iz * verts_x + ix) as u32;
            let i1 = (iz * verts_x + ix + 1) as u32;
            let i2 = ((iz + 1) * verts_x + ix) as u32;
            let i3 = ((iz + 1) * verts_x + ix + 1) as u32;
            indices.extend([i0, i2, i1, i1, i2, i3]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Creates your Mesh3d handle for a terrain chunk
/// Assumes you have a `Mesh3d` newtype around `Handle<Mesh>` in scope
pub fn create_heightmap_mesh3d(
    meshes: &mut Assets<Mesh>,
    region_sampler: &Res<RegionSampler>,
    chunk_size: f32,
    subdivisions: usize,
    world_offset: Vec2,
) -> Mesh3d {
    Mesh3d(meshes.add(generate_heightmap_mesh(
        region_sampler,
        chunk_size,
        subdivisions,
        world_offset,
    )))
}
