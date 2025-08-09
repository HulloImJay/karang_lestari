use bevy::prelude::Resource;
use noise::core::perlin::perlin_2d;
use noise::{NoiseFn, Perlin};

/// Resource wrapping a shared Perlin noise generator
#[derive(Resource)]
#[derive(Clone, Debug)]
pub struct HeightNoise {
    pub perlin_height: f64,
    pub perlin_scale: f64,
    pub perlin: Perlin,
    pub terrace_height: f64,
    pub terrace_steps: f64,
    pub terrace_scale: f64,
    pub terrace_source: Perlin,
    pub terrace_smooth_width :f32,
    pub offset:f64,
}

impl HeightNoise {
    pub fn sample(&self, point: [f64; 2]) -> f64 {
        let perlin_point_scaled = point.map(|coord| coord * self.perlin_scale);
        let perlin_val = self.perlin_height *(self.perlin.get(perlin_point_scaled));

        let terrace_point_scaled = point.map(|coord| coord * self.terrace_scale);
        let mut terrace_val = (self.terrace_source.get(terrace_point_scaled));
        terrace_val = smooth_terrace ((terrace_val * self.terrace_steps) as f32, self.terrace_smooth_width) as f64 / self.terrace_steps * self.terrace_height;
        perlin_val + terrace_val + self.offset
    }
}
/// Soft‐terrace a float so it “snaps” at integer steps
/// with a smooth ramp of width `w` (0.0 → hard terrace,
/// 1.0 → a full-step linear ramp).
///
/// - x: the input (e.g. `noise * num_steps`)
/// - w: ramp width in [0..1], recommended small (e.g. 0.1..0.3)
pub fn smooth_terrace(x: f32, w: f32) -> f32 {
    // integer terrace level
    let i = x.floor();
    // fractional [0..1)
    let f = x - i;
    // center the ramp around 0.5
    let lo = 0.5 - 0.5 * w;
    let hi = 0.5 + 0.5 * w;
    // normalize and clamp to [0..1]
    let mut t = (f - lo) / (hi - lo);
    t = t.clamp(0.0, 1.0);
    // cubic smoothstep: 3t^2 - 2t^3
    let s = t * t * (3.0 - 2.0 * t);
    // put us back on the “terraced” curve
    i + s
}
