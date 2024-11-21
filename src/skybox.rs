use nalgebra_glm::{Vec3, Vec4};
use rand::prelude::*;
use std::f32::consts::PI;
use crate::{Framebuffer, Uniforms};

pub struct Star {
    position: Vec3,
    brightness: f32,
}

pub struct Skybox {
    stars: Vec<Star>,
}

impl Skybox {
    pub fn new(star_count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut stars = Vec::with_capacity(star_count);

        for _ in 0..star_count {
            // Generate random spherical coordinates
            let theta = rng.gen::<f32>() * 2.0 * PI;  // Azimuth angle
            let phi = rng.gen::<f32>() * PI;          // Polar angle
            let radius = 100.0;  // Fixed radius for all stars

            // Convert spherical to Cartesian coordinates
            let x = radius * phi.sin() * theta.cos();
            let y = radius * phi.sin() * theta.sin();
            let z = radius * phi.cos();

            // Random brightness between 0.5 and 1.0
            let brightness = rng.gen::<f32>() * 0.5 + 0.5;

            stars.push(Star {
                position: Vec3::new(x, y, z),
                brightness,
            });
        }

        Skybox { stars }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer, uniforms: &Uniforms, camera_position: Vec3) {
        for star in &self.stars {
            // Calculate star position relative to camera
            let position = star.position + camera_position;
            
            // Project the star position to screen space
            let pos_vec4 = Vec4::new(position.x, position.y, position.z, 1.0);
            let projected = uniforms.projection_matrix * uniforms.view_matrix * pos_vec4;

            // Perform perspective division
            if projected.w <= 0.0 { continue; }
            let ndc = projected / projected.w;

            // Apply viewport transform
            let screen_pos = uniforms.viewport_matrix * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
            
            // Check if star is in front of camera and within screen bounds
            if screen_pos.z < 0.0 { continue; }
            
            let x = screen_pos.x as usize;
            let y = screen_pos.y as usize;
            
            if x < framebuffer.width && y < framebuffer.height {
                // Calculate star color based on brightness
                let intensity = (star.brightness * 255.0) as u8;
                let color = (intensity as u32) << 16 | (intensity as u32) << 8 | intensity as u32;
                
                framebuffer.set_current_color(color);
                framebuffer.point(x, y, 100.0);
            }
        }
    }
}
