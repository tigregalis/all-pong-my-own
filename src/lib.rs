use bevy::prelude::Vec3;
use rand::random;

/// get a random polar coordinate as a cartesian [`Vec3`] coordinate
pub fn random_polar(min_radius: f32, max_radius: f32, min_angle: f32, max_angle: f32) -> Vec3 {
    const TWO_PI: f32 = 2.0 * std::f32::consts::PI;
    debug_assert!(max_radius >= min_radius);
    debug_assert!(max_angle >= min_angle);
    let angle = min_angle + random::<f32>() * (max_angle - min_angle);
    let angle = angle * TWO_PI;
    let radius = min_radius + random::<f32>() * (max_radius - min_radius);
    Vec3::new(radius * angle.cos(), radius * angle.sin(), 0.0)
}
