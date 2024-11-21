use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3,
  pub pitch: f32,
  pub yaw: f32,
  pub roll: f32,
}

impl Camera {
  pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
    let forward = (center - eye).normalize();
    let pitch = (forward.y).asin();
    let yaw = forward.z.atan2(forward.x);
    
    Camera {
      eye,
      center,
      up: Vec3::new(0.0, 1.0, 0.0),
      pitch,
      yaw,
      roll: 0.0,
    }
  }

  pub fn move_forward(&mut self, amount: f32) {
    let forward = self.get_forward();
    self.eye += forward * amount;
    self.update_center();
  }

  pub fn move_up(&mut self, amount: f32) {
    self.eye += self.get_up() * amount;
    self.update_center();
  }

  pub fn rotate_yaw(&mut self, angle: f32) {
    self.yaw += angle;
    self.update_center();
  }

  pub fn rotate_pitch(&mut self, angle: f32) {
    self.pitch = (self.pitch + angle).clamp(-PI/2.0 + 0.1, PI/2.0 - 0.1);
    self.update_center();
  }

  pub fn set_roll(&mut self, angle: f32) {
    self.roll = angle;
  }

  pub fn get_forward(&self) -> Vec3 {
    Vec3::new(
      self.yaw.cos() * self.pitch.cos(),
      self.pitch.sin(),
      self.yaw.sin() * self.pitch.cos()
    ).normalize()
  }

  pub fn get_right(&self) -> Vec3 {
    self.get_forward().cross(&self.get_up()).normalize()
  }

  pub fn get_up(&self) -> Vec3 {
    Vec3::new(0.0, 1.0, 0.0)
  }

  fn update_center(&mut self) {
    let forward = self.get_forward();
    self.center = self.eye + forward;
  }
}
