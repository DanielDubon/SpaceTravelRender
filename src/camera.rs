use nalgebra_glm::Vec3;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct WarpState {
    pub start_position: Vec3,
    pub end_position: Vec3,
    pub start_direction: Vec3,
    pub end_direction: Vec3,
    pub progress: f32,
    pub duration: f32,
    pub is_active: bool,
}

impl WarpState {
    pub fn new() -> Self {
        WarpState {
            start_position: Vec3::new(0.0, 0.0, 0.0),
            end_position: Vec3::new(0.0, 0.0, 0.0),
            start_direction: Vec3::new(0.0, 0.0, -1.0),
            end_direction: Vec3::new(0.0, 0.0, -1.0),
            progress: 0.0,
            duration: 1.0,
            is_active: false,
        }
    }
}

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3,
  pub pitch: f32,
  pub yaw: f32,
  pub roll: f32,
  pub warp_state: WarpState,
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
      warp_state: WarpState::new(),
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

  pub fn start_warp(&mut self, target_pos: Vec3, target_direction: Vec3) {
    self.warp_state.start_position = self.eye;
    self.warp_state.end_position = target_pos;
    self.warp_state.start_direction = self.get_forward();
    self.warp_state.end_direction = target_direction;
    self.warp_state.progress = 0.0;
    self.warp_state.duration = 1.0; // 1 segundo de duración
    self.warp_state.is_active = true;
  }

  pub fn update_warp(&mut self, dt: f32) {
    if !self.warp_state.is_active {
        return;
    }

    self.warp_state.progress += dt / self.warp_state.duration;

    if self.warp_state.progress >= 1.0 {
        self.eye = self.warp_state.end_position;
        let direction = self.warp_state.end_direction;
        self.pitch = (direction.y).asin();
        self.yaw = direction.z.atan2(direction.x);
        self.roll = 0.0;
        self.warp_state.is_active = false;
        self.update_center();
        return;
    }

    // Función de suavizado
    let t = (self.warp_state.progress * std::f32::consts::PI).sin();
    
    // Interpolar posición
    self.eye = self.warp_state.start_position.lerp(
        &self.warp_state.end_position,
        t
    );

    // Interpolar dirección
    let direction = self.warp_state.start_direction.lerp(
        &self.warp_state.end_direction,
        t
    ).normalize();

    self.pitch = (direction.y).asin();
    self.yaw = direction.z.atan2(direction.x);
    
    // Efecto de roll durante el warp
    self.roll = (t * std::f32::consts::PI * 2.0).sin() * 0.5;
    
    self.update_center();
  }
}
