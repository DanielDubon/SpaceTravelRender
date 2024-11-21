use nalgebra_glm::{Vec3, Vec4, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions, KeyRepeat};
use std::f32::consts::PI;
mod skybox;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod planet_type;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use planet_type::PlanetType;
use skybox::Skybox;

pub struct CelestialBody {
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    shader_type: PlanetType,
    orbital_distance: f32,
    orbital_speed: f32,
    trail: Trail,
}

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite,
    camera_position: Vec3,
}

pub struct Spaceship {
    model: Obj,
    scale: f32,
    offset: Vec3,
}

pub struct TrailParticle {
    position: Vec3,
    color: u32,
    lifetime: f32,
    size: f32,
}

pub struct Trail {
    particles: Vec<TrailParticle>,
    max_particles: usize,
    spawn_timer: f32,
}

impl Trail {
    fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
            spawn_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.particles.retain_mut(|particle| {
            particle.lifetime -= dt;
            particle.size *= 0.999;
            particle.lifetime > 0.0
        });
    }

    fn add_particle(&mut self, position: Vec3, color: u32, is_moon: bool) {
        if self.particles.len() >= self.max_particles {
            self.particles.remove(0);
        }

        let lifetime = if is_moon { 2.0 } else { 200000.0 };
        let size = if is_moon { 0.2 } else { 0.50000 };

        self.particles.push(TrailParticle {
            position,
            color,
            lifetime,
            size,
        });
    }
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise() 
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_cell_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_frequency(Some(0.1));
    noise
}

fn create_ground_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    
   
    noise.set_noise_type(Some(NoiseType::Cellular)); 
    noise.set_fractal_type(Some(FractalType::FBm)); 
    noise.set_fractal_octaves(Some(5));              
    noise.set_fractal_lacunarity(Some(2.0));         
    noise.set_fractal_gain(Some(0.5));               
    noise.set_frequency(Some(0.05));                 

    noise
}

fn create_lava_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(42);
    
 
    noise.set_noise_type(Some(NoiseType::Perlin));  
    noise.set_fractal_type(Some(FractalType::FBm)); 
    noise.set_fractal_octaves(Some(6));            
    noise.set_fractal_lacunarity(Some(2.0));       
    noise.set_fractal_gain(Some(0.5));              
    noise.set_frequency(Some(0.002));                
    
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    planet_type: &PlanetType
) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Shader Stage
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let depth = if matches!(planet_type, PlanetType::Spaceship) {
                // Forzar que la nave siempre esté al frente
                -1.0
            } else {
                fragment.depth
            };

            if framebuffer.should_draw(x, y, depth) {
                let shaded_color = fragment_shader(&fragment, &uniforms, planet_type);
                let color = shaded_color.to_hex();
                framebuffer.set_current_color(color);
                framebuffer.point(x, y, depth);
            }
        }
    }
}

fn calculate_detail_level(distance: f32) -> usize {
    if distance < 5.0 {
        0  // Máximo detalle
    } else if distance < 20.0 {
        1  // Detalle medio
    } else {
        2  // Bajo detalle
    }
}

fn get_lod_mesh(vertex_arrays: &[Vertex], detail_level: usize) -> &[Vertex] {
    // Por ahora, retornamos el mismo mesh para todos los niveles
    vertex_arrays
}

struct Frustum {
    fov: f32,
    near: f32,
    far: f32,
    aspect: f32,
}

impl Frustum {
    fn new(fov: f32, near: f32, far: f32, aspect: f32) -> Self {
        Self {
            fov,
            near,
            far,
            aspect,
        }
    }

    fn is_visible(&self, camera_pos: &Vec3, camera_forward: &Vec3, object_pos: &Vec3, object_radius: f32) -> bool {
        // Verificar distancia
        let to_object = object_pos - camera_pos;
        let distance = to_object.magnitude();
        
        // Si está muy cerca o muy lejos, no renderizar
        if distance < self.near || distance > self.far {
            return false;
        }

        // Verificar si está dentro del campo de visión
        let direction = to_object.normalize();
        let angle = camera_forward.dot(&direction).acos();
        
        // Convertir FOV a radianes y comparar
        let half_fov = (self.fov * std::f32::consts::PI / 180.0) / 2.0;
        
        // Añadir el radio del objeto al ángulo de visión
        let apparent_angle = half_fov + (object_radius / distance).asin();
        
        angle <= apparent_angle
    }
}

fn check_collision(position: &Vec3, celestial_bodies: &[CelestialBody]) -> bool {
    for body in celestial_bodies {
        let distance = (position - body.position).magnitude();
        let collision_radius = body.scale * 2.0;
        
        if distance < collision_radius {
            return true; // Hay colisión
        }
    }
    false // No hay colisión
}

fn handle_input(window: &Window, camera: &mut Camera, celestial_bodies: &[CelestialBody]) {
    let movement_speed = 0.2;
    let rotation_speed = PI/128.0;
    let bank_angle = PI/16.0;

    // Añadir warping a planetas específicos con KeyRepeat::No
    if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
        warp_to_planet(camera, &celestial_bodies[0], 8.0); // Sol (más lejos por ser más grande)
    } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
        warp_to_planet(camera, &celestial_bodies[3], 3.0); // Tierra
    } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
        warp_to_planet(camera, &celestial_bodies[5], 5.0); // Júpiter (más lejos por ser grande)
    } else if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
        warp_to_planet(camera, &celestial_bodies[10], 12.0); // Agujero Negro (mucho más lejos)
    }

    // Calcular la nueva posición antes de aplicarla
    let mut new_position = camera.eye;

    // Movimiento lateral con rotación
    if window.is_key_down(Key::A) {
        camera.rotate_yaw(-rotation_speed);
        camera.set_roll(bank_angle);
    } else if window.is_key_down(Key::D) {
        camera.rotate_yaw(rotation_speed);
        camera.set_roll(-bank_angle);
    } else {
        camera.set_roll(camera.roll * 0.9);
    }

    // Control de pitch
    if window.is_key_down(Key::Up) {
        camera.rotate_pitch(rotation_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.rotate_pitch(-rotation_speed);
    }

    // Calcular el movimiento deseado
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    
    if window.is_key_down(Key::W) {
        movement += camera.get_forward() * movement_speed;
    }
    if window.is_key_down(Key::S) {
        movement += camera.get_forward() * (-movement_speed * 0.5);
    }
    if window.is_key_down(Key::Q) {
        movement += camera.get_up() * (movement_speed * 0.7);
    }
    if window.is_key_down(Key::E) {
        movement += camera.get_up() * (-movement_speed * 0.7);
    }

    // Verificar colisiones antes de aplicar el movimiento
    new_position += movement;
    
    if !check_collision(&new_position, celestial_bodies) {
        camera.eye = new_position;
        camera.center = camera.eye + camera.get_forward();
    }
}

fn warp_to_planet(camera: &mut Camera, body: &CelestialBody, distance: f32) {
    // Calcular la posición relativa considerando el movimiento orbital
    let offset = Vec3::new(0.0, 0.0, distance + body.scale);
    
    // Usar la posición actual del planeta
    let target_pos = body.position + offset;
    
    // Calcular la dirección mirando hacia el planeta
    let target_direction = (body.position - target_pos).normalize();
    
    // Iniciar el warp con la posición y dirección calculadas
    camera.start_warp(target_pos, target_direction);
}

fn render_trail(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    particle: &TrailParticle,
) {
    let model_matrix = create_model_matrix(
        particle.position,
        particle.size,
        Vec3::new(0.0, 0.0, 0.0)
    );

    let position_clip = uniforms.projection_matrix * uniforms.view_matrix * model_matrix * Vec4::new(0.0, 0.0, 0.0, 1.0);
    
    if position_clip.w <= 0.0 {
        return;
    }

    let position_ndc = Vec3::new(
        position_clip.x / position_clip.w,
        position_clip.y / position_clip.w,
        position_clip.z / position_clip.w,
    );

    let position_screen = uniforms.viewport_matrix * Vec4::new(
        position_ndc.x,
        position_ndc.y,
        position_ndc.z,
        1.0,
    );

    let x = position_screen.x as usize;
    let y = position_screen.y as usize;

    if x < framebuffer.width && y < framebuffer.height {
        let alpha = (particle.lifetime * 255.0) as u32;
        let color = (particle.color & 0x00FFFFFF) | (alpha << 24);
        
        framebuffer.set_current_color(color);
        framebuffer.point(x, y, position_screen.z);
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Renderer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
        .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x000000);

    
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let obj = Obj::load("assets/models/esfera.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array(); 
    let mut time = 0;
    let skybox = Skybox::new(1000);

    let noise = create_noise();
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    let mut uniforms = Uniforms { 
        model_matrix: Mat4::identity(), 
        view_matrix: Mat4::identity(), 
        projection_matrix, 
        viewport_matrix, 
        time: 0, 
        noise,
        camera_position: camera.eye,
    };

    
    let mut celestial_bodies = vec![
        CelestialBody {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: 2.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Sun,
            orbital_distance: 0.0,
            orbital_speed: 0.0,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(6.0, 0.0, 0.0),
            scale: 0.4,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Mercury,
            orbital_distance: 12.0,
            orbital_speed: 0.002,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(12.0, 0.0, 0.0),
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Venus,
            orbital_distance: 24.0,
            orbital_speed: 0.0015,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(18.0, 0.0, 0.0),
            scale: 0.7,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Earth,
            orbital_distance: 36.0,
            orbital_speed: 0.001,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(24.0, 0.0, 0.0),
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Mars,
            orbital_distance: 48.0,
            orbital_speed: 0.0008,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(32.0, 0.0, 0.0),
            scale: 1.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Jupiter,
            orbital_distance: 64.0,
            orbital_speed: 0.0005,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(40.0, 0.0, 0.0),
            scale: 1.3,
            rotation: Vec3::new(0.2, 0.0, 0.0),
            shader_type: PlanetType::Saturn,
            orbital_distance: 80.0,
            orbital_speed: 0.0004,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(48.0, 0.0, 0.0),
            scale: 0.9,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Uranus,
            orbital_distance: 96.0,
            orbital_speed: 0.0003,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(56.0, 0.0, 0.0),
            scale: 0.9,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Neptune,
            orbital_distance: 102.0,
            orbital_speed: 0.0002,
            trail: Trail::new(50000),
        },
        CelestialBody {
            position: Vec3::new(18.0, 0.0, 2.0),
            scale: 0.2,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Moon,
            orbital_distance: 2.0,
            orbital_speed: 0.03,
            trail: Trail::new(50),
        },
        CelestialBody {
            position: Vec3::new(-20.0, 0.0, -20.0),
            scale: 4.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::BlackHole,
            orbital_distance: 280.0,
            orbital_speed: 0.0001,
            trail: Trail::new(50000),
        },
    ];

    // Cargar el modelo de la nave (asegúrate de tener un modelo .obj de una nave)
    let spaceship = Spaceship {
        model: Obj::load("assets/models/nave.obj").expect("Failed to load spaceship"),
        scale: 0.02,
        offset: Vec3::new(0.0, -0.1, -1.0),
    };
    let spaceship_vertices = spaceship.model.get_vertex_array();

    let frustum = Frustum::new(
        45.0,           // FOV en grados
        0.1,            // Near plane
        1000.0,         // Far plane
        window_width as f32 / window_height as f32  // Aspect ratio
    );

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;
        
        // Actualizar la cámara antes de manejar el input
        camera.update_warp(0.016); // 60 FPS aproximadamente
        handle_input(&window, &mut camera, &celestial_bodies);
        
        framebuffer.clear();
        
        // 1. Primero renderizar el skybox (fondo)
        skybox.render(&mut framebuffer, &uniforms, camera.eye);

        uniforms.camera_position = camera.eye;  // Actualizar posición de la cámara
        let camera_forward = camera.get_forward();

        // Renderizar planetas con culling
        for body in &celestial_bodies {
            let apparent_radius = body.scale * 2.0;
            
            if frustum.is_visible(&camera.eye, &camera_forward, &body.position, apparent_radius) {
                uniforms.model_matrix = create_model_matrix(
                    body.position,
                    body.scale,
                    body.rotation + Vec3::new(0.0, time as f32 * 0.01, 0.0)
                );
                uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
                uniforms.time = time;
                
                render(&mut framebuffer, &uniforms, &vertex_arrays, &body.shader_type);
            }
        }

        // 3. Finalmente la nave (siempre al final para que esté encima)
        let ship_position = camera.eye 
            + camera.get_forward() * spaceship.offset.z 
            + camera.get_up() * spaceship.offset.y
            + camera.get_right() * spaceship.offset.x;
        
        uniforms.model_matrix = create_model_matrix(
            ship_position,
            spaceship.scale,
            Vec3::new(
                0.0,          // No aplicamos pitch para mantener la nave nivelada
                -camera.yaw + PI * 1.5,   // Combinamos las rotaciones (90° + 180° = 270° = 3PI/2)
                camera.roll   // Mantenemos el roll para la inclinación en los giros
            )
        );
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        
        // Asegurarnos de que la nave siempre esté en frente
        render(&mut framebuffer, &uniforms, &spaceship_vertices, &PlanetType::Spaceship);

        // Actualizar posiciones de los planetas
        let earth_position = celestial_bodies.iter()
            .find(|b| matches!(b.shader_type, PlanetType::Earth))
            .map(|earth| earth.position)
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));

        for body in &mut celestial_bodies {
            match body.shader_type {
                PlanetType::Sun => (), // El sol no se mueve
                PlanetType::Moon => {
                    let moon_angle = time as f32 * body.orbital_speed;
                    body.position = earth_position + Vec3::new(
                        body.orbital_distance * moon_angle.cos(),
                        0.0,
                        body.orbital_distance * moon_angle.sin()
                    );
                },
                PlanetType::BlackHole => {
                    let angle = time as f32 * body.orbital_speed;
                    body.position.x = body.orbital_distance * angle.cos();
                    body.position.z = body.orbital_distance * angle.sin();
                },
                _ => {
                    let angle = time as f32 * body.orbital_speed;
                    body.position.x = body.orbital_distance * angle.cos();
                    body.position.z = body.orbital_distance * angle.sin();
                }
            }
        }

        // Primero renderizar las estelas
        for body in &celestial_bodies {
            for particle in &body.trail.particles {
                render_trail(&mut framebuffer, &uniforms, particle);
            }
        }

        // Actualizar las estelas al final del frame
        for body in &mut celestial_bodies {
            body.trail.update(0.016);
            
            let color = match body.shader_type {
                PlanetType::Sun => 0xFFFFAA00,     // Naranja brillante
                PlanetType::Mercury => 0xFFAA8866,  // Marrón claro
                PlanetType::Venus => 0xFFFFCC99,    // Amarillo pálido
                PlanetType::Earth => 0xFF0066FF,    // Azul brillante
                PlanetType::Mars => 0xFFFF3300,     // Rojo anaranjado
                PlanetType::Jupiter => 0xFFFFAA66,  // Naranja suave
                PlanetType::Saturn => 0xFFFFCC66,   // Dorado
                PlanetType::Uranus => 0xFF66FFFF,   // Cyan claro
                PlanetType::Neptune => 0xFF0066FF,  // Azul profundo
                PlanetType::Moon => 0xFFCCCCCC,     // Gris claro
                PlanetType::BlackHole => 0xFF440044, // Púrpura oscuro
                PlanetType::Spaceship => 0xFFFFFFFF, // Blanco
            };
            
            let is_moon = matches!(body.shader_type, PlanetType::Moon);
            body.trail.add_particle(body.position, color, is_moon);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}

