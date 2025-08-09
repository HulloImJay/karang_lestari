use crate::height_noise::HeightNoise;
use bevy::color::Color;
use bevy::prelude::Resource;
use glam::{Quat, Vec2, Vec3};
use rand::Rng;

/// Your region definition; add fields here as you go.
#[derive(Clone, Debug)]
pub struct Region {
    pub name: String,
    pub weight: u32,
    pub height_sampler: HeightNoise,
    pub objects: Vec<ObjectSelection>,
    pub lighting_setups: Vec<LightingSetup>,
    objects_prefix: Vec<u32>,
    objects_total_weight: u32,
}

#[derive(Clone, Debug)]
pub struct LightingSetup {
    pub name:String,
    pub primary_color: Color,
    pub primary_illuminance: f32,
    pub secondary_color: Color,
    pub secondary_illuminance: f32,
    pub time:f32,
    pub fog_colour:Color,
    pub clear_colour:Color,
}

impl Region {
    pub fn new(
        name: String,
        weight: u32,
        height_sampler: HeightNoise,
        objects: Vec<ObjectSelection>,
        lighting_setups: Vec<LightingSetup>,
    ) -> Region {
        let mut objects_prefix = Vec::with_capacity(objects.len());
        let mut objects_total_weight = 0;
        for o in &objects {
            objects_total_weight += o.selection_weight;
            objects_prefix.push(objects_total_weight);
        }

        Region {
            name,
            weight,
            height_sampler,
            objects,
            objects_prefix,
            objects_total_weight,
            lighting_setups,
        }
    }
}

impl Default for Region {
    fn default() -> Self {
        Region {
            name: "".to_string(),
            weight: 0,
            height_sampler: HeightNoise {
                perlin_height: 0.0,
                perlin_scale: 0.0,
                perlin: Default::default(),
                terrace_height: 0.0,
                terrace_steps: 0.0,
                terrace_scale: 0.0,
                terrace_source: Default::default(),
                terrace_smooth_width: 0.0,
                offset: 0.0,
            },
            objects: vec![],
            objects_prefix: vec![],
            objects_total_weight: 0,
            lighting_setups: vec![],
        }
    }
}

impl Region {
    pub fn pick_object(&self) -> String {
        let mut rng = rand::rng();
        let r = rng.random_range(0..self.objects_total_weight);

        let target = r + 1;
        let id = self
            .objects_prefix
            .binary_search(&target)
            .unwrap_or_else(|i| i);

        self.objects[id].name.clone()
    }
}

#[derive(Clone, Debug)]
pub struct ObjectSelection {
    pub name: String,
    pub selection_weight: u32,
}

/// A sampler that, given any world‐pos `p: Vec2`, returns
/// `(near_id, far_id, blend)` exactly as described.
#[derive(Resource)]
pub struct RegionSampler {
    pub regions: Vec<Region>,
    prefix: Vec<u32>,
    total_weight: u32,
    cell_size: f32,
    jitter: f32,
    blend_dist: f32,
    seed: u64,
}

impl RegionSampler {
    /// Build from a list of regions (with weights), plus your
    /// cell_size, jitter, blend width and seed.
    pub fn new(
        mut regions: Vec<Region>,
        cell_size: f32,
        jitter: f32,
        blend_dist: f32,
        seed: u64,
    ) -> Self {
        // build prefix sums of weights
        let mut prefix = Vec::with_capacity(regions.len());
        let mut acc = 0;
        for r in &regions {
            acc += r.weight;
            prefix.push(acc);
        }
        let total_weight = acc;

        RegionSampler {
            regions,
            prefix,
            total_weight,
            cell_size,
            jitter,
            blend_dist,
            seed,
        }
    }

    /// Sample your biomes: returns (id1, id2, t)
    /// where id1/id2 are the two nearest region‐IDs,
    /// and t∈[0,1] is how much it blends toward id2.
    pub fn sample_region(&self, p: Vec2) -> ([usize; 3], [f32; 3]) {
        // which grid cell we're in
        let cx = (p.x / self.cell_size).floor() as i32;
        let cy = (p.y / self.cell_size).floor() as i32;

        // track closest / second‐closest
        let mut best1 = f32::INFINITY;
        let mut best2 = f32::INFINITY;
        let mut best3 = f32::INFINITY;
        let mut id1 = 0;
        let mut id2 = 0;
        let mut id3 = 0;

        // search the 3×3 neighbors
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cell_x = cx + dx;
                let cell_y = cy + dy;

                // ==== JITTERED SITE POSITION ====
                let h_j = self.hash(cell_x, cell_y, self.seed);
                let fx = (h_j as f32 / u64::MAX as f32) * 2.0 - 1.0;
                let fy = (((h_j >> 32) as f32 / u64::MAX as f32) * 2.0) - 1.0;
                let site = Vec2::new(
                    (cell_x as f32 + 0.5 + fx * self.jitter) * self.cell_size,
                    (cell_y as f32 + 0.5 + fy * self.jitter) * self.cell_size,
                );

                let dist = (p - site).length_squared();

                // ==== WEIGHTED REGION PICK ====
                // use a second hash (tweak seed) for region choice
                let h_r = self.hash(cell_x, cell_y, self.seed ^ 0x9E3779B97f4A7C15);
                let r = (h_r % self.total_weight as u64) as u32;
                // find first prefix > r  (i.e. bucket search)
                let target = r + 1;
                let region_id = self.prefix.binary_search(&target).unwrap_or_else(|i| i);

                // ==== KEEP TWO NEAREST SITES ====
                if dist < best1 {
                    best3 = best2;
                    id3 = id2;
                    best2 = best1;
                    id2 = id1;
                    best1 = dist;
                    id1 = region_id;
                } else if dist < best2 {
                    best3 = best2;
                    id3 = id2;
                    best2 = dist;
                    id2 = region_id;
                } else if dist < best3 {
                    best3 = dist;
                    id3 = region_id;
                }
            }
        }

        // // smooth blend based on gap between √dists
        // let gap = best2.sqrt() - best1.sqrt();
        // let t1 = (gap / self.blend_dist).clamp(0.0, 1.0);
        //
        // // t1 gives:
        // //  0.0 for mid-way between best1 and best2
        // //  1.0 for fully in best1
        // // so we remap to the more easy-to-understand
        // //  0.0 for best1 and
        // //  1.0 for best2
        // let t = (1.0 - t1) * 0.5;

        // (id1, id2, t)

        // convert squared→actual distances
        let d1 = best1.sqrt();
        let d2 = best2.sqrt();
        let d3 = best3.sqrt();

        // compute how “far” each is from the winner
        let delta_1 = 0.0; // d1−d1
        let delta_2 = d2 - d1;
        let delta_3 = d3 - d1;

        // raw weights fall off linearly over blend_dist…
        let w1 = (self.blend_dist - delta_1).clamp(0.0, self.blend_dist);
        let w2 = (self.blend_dist - delta_2).clamp(0.0, self.blend_dist);
        let w3 = (self.blend_dist - delta_3).clamp(0.0, self.blend_dist);

        // …then normalize so they sum to 1
        let sum = w1 + w2 + w3;
        let w1 = w1 / sum;
        let w2 = w2 / sum;
        let w3 = w3 / sum;

        ([id1, id2, id3], [w1, w2, w3])
    }

    pub fn sample_surface_height(&self, p: Vec2) -> f64 {
        let ([id1, id2, id3], [w1, w2, w3]) = self.sample_region(p);
        let p_f64 = [(p.x) as f64, (p.y) as f64];
        let height1 = self.regions[id1].height_sampler.sample(p_f64);
        let height2 = self.regions[id2].height_sampler.sample(p_f64);
        let height3 = self.regions[id3].height_sampler.sample(p_f64);
        // print!("t: {}", t);
        // height1.lerp(height2, t as f64)
        // (t * 3.0) as f64
        // (id1 as f64).lerp((id2 as f64), t as f64)
        // ((id1 as f32) * w1 + (id2 as f32) * w2 + (id3 as f32) * w3) as f64
        height1 * w1 as f64 + height2 * w2 as f64 + height3 * w3 as f64
    }

    pub fn sample_surface_orientation(
        &self,
        p: Vec2, // .x = X, .y = Z
        eps: f32,
    ) -> (Quat, Vec3) {
        // helper to sample and cast
        let sample = |dx: f32, dz: f32| self.sample_surface_height(p + Vec2::new(dx, dz)) as f32;

        // sample left/right/up/down
        let hl = sample(-eps, 0.0);
        let hr = sample(eps, 0.0);
        let hd = sample(0.0, -eps);
        let hu = sample(0.0, eps);

        // approximate partial derivatives
        let dh_dx = (hr - hl) / (2.0 * eps);
        let dh_dz = (hu - hd) / (2.0 * eps);

        // build the normal vector and normalize
        let normal = Vec3::new(-dh_dx, 1.0, -dh_dz).normalize();

        // rotate Y up → this normal
        (Quat::from_rotation_arc(Vec3::Y, normal), normal)
    }

    /// A simple 2D→u64 mixer. You can swap in any small
    /// xorshift/SplitMix variant here.
    fn hash(&self, x: i32, y: i32, seed: u64) -> u64 {
        let mut h = seed.wrapping_add(x as u64).wrapping_mul(0x9E3779B97f4A7C15);
        h = h.rotate_left(31) ^ y as u64;
        h = h.wrapping_mul(0x9E3779B97f4A7C15);
        h.rotate_left(31)
    }
}
