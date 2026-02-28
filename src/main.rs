// Import all modules from the macroquad crate into the current scope.
// macroquad provides low-level functions for game development: windowing, input, 2D/3D graphics, etc.
use macroquad::prelude::*;

// Define an enumeration to keep track of the current camera perspective.
// We use #[derive(Debug)] to allow the enum to be formatted as a string for display messages.
#[derive(Debug)]
enum CameraMode {
    // FirstPerson mode: The camera is placed near the pet's "head" looking forward.
    FirstPerson,
    // ThirdPerson mode: The camera follows behind the pet from a distance.
    ThirdPerson,
}

// The Pet struct represents the main entity in our simulation.
// It stores all attributes related to its state, position, and movement.
struct Pet {
    // The user-defined name of the pet.
    name: String,
    // hunger: A value from 0.0 to 100.0. Higher means the pet is hungrier.
    hunger: f32,
    // happiness: A value from 0.0 to 100.0. Higher means the pet is happier.
    happiness: f32,
    // energy: A value from 0.0 to 100.0. Higher means the pet has more energy.
    energy: f32,
    // Boolean flag to track if the pet is currently "alive" (game over state).
    is_alive: bool,
    // Stores the timestamp of the last time the pet's stats were automatically updated.
    last_update: f64,
    // x: World coordinate for horizontal position.
    x: f32,
    // y: World coordinate for vertical position (elevation). Currently unused for movement.
    y: f32,
    // z: World coordinate for depth position (forward/backward).
    z: f32,
    // vx: Velocity along the X-axis (horizontal).
    vx: f32,
    // vz: Velocity along the Z-axis (depth).
    vz: f32,
    // rotation_v: Tracks vertical rotation to simulate the pet rolling forward or backward.
    rotation_v: f32,
    // pitch: Vertical look angle for the FirstPerson camera (looking up/down).
    pitch: f32,
    // yaw: Horizontal rotation angle for movement and camera direction.
    yaw: f32,
    // is_stationary: Boolean flag to track if the pet is currently not moving.
    // This is required for actions like feeding, playing, and sleeping.
    is_stationary: bool,
    // The timestamp when this pet was created/started its current life.
    start_time: f64,
    // The timestamp when this pet passed away.
    death_time: Option<f64>,
}

// The Bug struct represents an enemy that chases the pet.
struct Bug {
    // Current X-coordinate of the bug.
    x: f32,
    // Current Z-coordinate of the bug.
    z: f32,
    // Movement speed of the bug.
    speed: f32,
}

impl Bug {
    // Constructor to create a new Bug at a specific position.
    fn new(x: f32, z: f32) -> Bug {
        Bug { x, z, speed: 0.12 }
    }

    // Update function to move the bug towards the pet.
    fn update(&mut self, pet_x: f32, pet_z: f32) {
        // Calculate the direction vector towards the pet.
        let dx = pet_x - self.x;
        let dz = pet_z - self.z;
        let dist = (dx * dx + dz * dz).sqrt();
        
        // Normalize the vector and move the bug if it's not already at the pet's position.
        if dist > 0.1 {
            self.x += (dx / dist) * self.speed;
            self.z += (dz / dist) * self.speed;
        }
    }
}

// The Ball struct represents an interactive physics object in the world.
struct Ball {
    // Current X-coordinate of the ball.
    x: f32,
    // Current Z-coordinate of the ball.
    z: f32,
    // Current velocity of the ball on the X-axis.
    vx: f32,
    // Current velocity of the ball on the Z-axis.
    vz: f32,
    // Tracks the rotation of the ball to visualize rolling on its surface.
    rotation: f32,
}

// Implement methods for the Ball struct.
impl Ball {
    // Constructor to create a new Ball with default starting values.
    fn new() -> Ball {
        Ball {
            // Initial position offset from the world origin.
            x: 5.0,
            z: 5.0,
            // Starts stationary with no initial velocity.
            vx: 0.0,
            vz: 0.0,
            // Starts with zero rotation.
            rotation: 0.0,
        }
    }

    // Update function to be called every frame to process ball physics.
    fn update(&mut self) {
        // Apply friction to the velocities. This simulates air/ground resistance.
        // Multiplying by 0.95 reduces the speed by 5% every single frame.
        self.vx *= 0.95;
        self.vz *= 0.95;

        // Update the position of the ball based on its current velocity.
        // This is a basic Euler integration: Position += Velocity.
        self.x += self.vx;
        self.z += self.vz;

        // Calculate the speed (magnitude of the velocity vector).
        let speed = (self.vx * self.vx + self.vz * self.vz).sqrt();
        // Increment the rotation based on how fast the ball is moving.
        // This creates the visual effect of the ball rolling on the ground.
        self.rotation += speed * 5.0;

        // Since the world is infinite, we no longer need to check for world boundaries here.
    }
}

// Implement methods for the Pet struct.
impl Pet {
    // Constructor to initialize a new Pet with a name and starting stats.
    fn new(name: String) -> Pet {
        Pet {
            name,
            // Initialize stats to 50% capacity.
            hunger: 50.0,
            happiness: 50.0,
            energy: 50.0,
            // Start the pet in an alive state.
            is_alive: true,
            // Record the current time to start the stat degradation timer.
            last_update: get_time(),
            // Start at the world origin (0,0,0).
            x: 0.0,
            y: 0.0,
            z: 0.0,
            // No initial movement or rotation.
            vx: 0.0,
            vz: 0.0,
            rotation_v: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            is_stationary: true,
            // Record the current time to start the survival timer for this pet.
            start_time: get_time(),
            // Initially, the pet hasn't died.
            death_time: None,
        }
    }

    // Update function to handle input, movement, and stat changes every frame.
    fn update(&mut self, is_fps: bool) {
        // If the pet has already passed away, we skip all logic updates.
        if !self.is_alive {
            return;
        }

        // Check if any movement keys are being pressed this frame.
        // We do this check early so we can use it for stat updates.
        let is_key_moving = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) ||
                            is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) ||
                            is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) ||
                            is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        
        // Also check if the pet has significant velocity.
        let velocity_sq = self.vx * self.vx + self.vz * self.vz;
        self.is_stationary = !is_key_moving && velocity_sq < 0.0001;

        let is_moving = is_key_moving;

        // Get the current time since the program started.
        let now = get_time();
        // Check if at least 1 second has passed since the last stat update.
        // This makes the stats decay over time rather than every frame.
        if now - self.last_update > 1.0 {
            // Every second, the pet gets hungrier, less happy.
            self.hunger += 1.0;
            self.happiness -= 0.25;
            
            // If the pet is NOT moving (idle/afk), it gains 1 energy per second.
            if !is_moving {
                self.energy += 1.0;
            } else {
                // Moving consumes energy.
                self.energy -= 0.5;
            }

            // Reset the timer.
            self.last_update = now;
        }

        // Apply friction to the pet's movement (same as the ball).
        self.vx *= 0.95;
        self.vz *= 0.95;

        // Acceleration constant: how much velocity is added per frame when moving.
        let accel = 0.02;
        // Rotation speed constant: how fast the pet turns left or right.
        let rotation_speed = 0.05;

        // Track if the pet actually moved this frame (for animation).
        let mut actually_moved = false;

        // If the camera is in FirstPerson mode, we use the mouse to look around.
        if is_fps {
            // Get the change in mouse position since the last frame.
            let m_delta = mouse_delta_position();
            // Sensitivity multiplier for the mouse look.
            let sensitivity = 3.0;
            // Update yaw based on horizontal mouse movement.
            self.yaw += m_delta.x * sensitivity;
            // Update pitch based on vertical mouse movement.
            self.pitch -= m_delta.y * sensitivity;
            // Clamp pitch to prevent the player from looking 360 degrees vertically.
            // 1.5 radians is approximately 85 degrees.
            self.pitch = self.pitch.clamp(-1.5, 1.5);
        }

        // Handle forward and backward movement based on the current yaw direction.
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            // Use trigonometry to calculate the forward movement vector.
            self.vx += self.yaw.sin() * accel;
            self.vz += self.yaw.cos() * accel;
            actually_moved = true;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            // Moving backward: subtract the forward vector.
            self.vx -= self.yaw.sin() * accel;
            self.vz -= self.yaw.cos() * accel;
            actually_moved = true;
        }

        // Handle turning (ThirdPerson) or strafing (FirstPerson).
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            if is_fps {
                // In FPS mode, 'A' strafes left (moving perpendicular to the look direction).
                // Subtracting PI/2 (1.5708 radians) from yaw shifts the direction by -90 degrees.
                self.vx += (self.yaw - 1.5708).sin() * accel;
                self.vz += (self.yaw - 1.5708).cos() * accel;
                actually_moved = true;
            } else {
                // In ThirdPerson mode, 'A' rotates the pet to the left.
                self.yaw -= rotation_speed;
            }
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            if is_fps {
                // In FPS mode, 'D' strafes right.
                // Adding PI/2 (1.5708 radians) to yaw shifts the direction by 90 degrees.
                self.vx += (self.yaw + 1.5708).sin() * accel;
                self.vz += (self.yaw + 1.5708).cos() * accel;
                actually_moved = true;
            } else {
                // In ThirdPerson mode, 'D' rotates the pet to the right.
                self.yaw += rotation_speed;
            }
        }

        // Apply the calculated velocities to the pet's world position.
        self.x += self.vx;
        self.z += self.vz;

        // Calculate the magnitude of movement to determine the rolling speed.
        let speed = (self.vx * self.vx + self.vz * self.vz).sqrt();
        // Calculate the dot product between velocity and look direction.
        // This helps determine if the pet is moving "forward" or "backward" relative to its face.
        let dot = self.vx * self.yaw.sin() + self.vz * self.yaw.cos();
        
        // Update the vertical rotation (rolling) based on movement direction.
        if dot > 0.0 {
            // Moving forward: roll one way.
            self.rotation_v += speed * 5.0;
        } else {
            // Moving backward: roll the other way.
            self.rotation_v -= speed * 5.0;
        }

        // If the pet is moving, update its stats slightly.
        if actually_moved {
            // Consumes energy and increases hunger when moving.
            self.energy -= 0.05;
            self.hunger += 0.02;
        }

        // Game Over Condition: if the pet gets too hungry or runs out of energy.
        if self.hunger >= 100.0 || self.energy <= 0.0 {
            self.is_alive = false;
            // Record the exact time of death to freeze the survival timer.
            if self.death_time.is_none() {
                self.death_time = Some(get_time());
            }
        }

        // Clamp stats between 0.0 and 100.0 to keep them within valid ranges for the HUD.
        self.hunger = self.hunger.clamp(0.0, 100.0);
        self.happiness = self.happiness.clamp(0.0, 100.0);
        self.energy = self.energy.clamp(0.0, 100.0);
    }

    // Method to reduce hunger when feeding the pet.
    fn feed(&mut self) -> bool {
        if self.is_alive && self.is_stationary {
            // Feeding reduces hunger significantly.
            self.hunger -= 15.0;
            return true;
        }
        false
    }

    // Method to increase happiness by playing.
    fn play(&mut self) -> bool {
        if self.is_alive && self.is_stationary {
            // Playing increases happiness but costs energy.
            self.happiness += 15.0;
            self.energy -= 10.0;
            return true;
        }
        false
    }

    // Method to restore energy by sleeping.
    fn sleep(&mut self) -> bool {
        if self.is_alive && self.is_stationary {
            // Sleeping restores energy but time passes, making the pet slightly hungrier.
            self.energy += 20.0;
            self.hunger += 5.0;
            return true;
        }
        false
    }
}

// Utility function to draw a progress bar on the screen (UI/HUD).
fn draw_bar(y: f32, label: &str, value: f32, color: Color) {
    // Draw the text label for the stat.
    draw_text(label, 20.0, y + 15.0, 20.0, DARKGRAY);
    // Draw the background of the bar (the gray track).
    draw_rectangle(120.0, y, 200.0, 20.0, LIGHTGRAY);
    // Draw the filled portion of the bar representing the stat value.
    // The width is 'value * 2.0' because the max value is 100 and the bar width is 200.
    draw_rectangle(120.0, y, value * 2.0, 20.0, color);
}

// Entry point of the application using the macroquad::main macro.
#[macroquad::main("Rust Pet Sim 3D")]
async fn main() {
    // Initialize the pet with an empty name.
    let mut my_pet = Pet::new("".to_string());
    // String buffer to store the user's input during the naming phase.
    let mut naming_input = String::new();
    // State flag to track if we are done with the naming screen.
    let mut naming_done = false;

    // Naming screen loop: runs before the actual game starts.
    while !naming_done {
        // Clear the screen with a light gray color.
        clear_background(LIGHTGRAY);
        // Display instructions and current input.
        draw_text("Name your 3D pet:", 20.0, 100.0, 30.0, BLACK);
        draw_text(&naming_input, 20.0, 150.0, 40.0, BLUE);
        draw_text("Press ENTER to start", 20.0, 200.0, 20.0, DARKGRAY);

        // Capture keyboard characters to build the pet's name.
        if let Some(c) = get_char_pressed() {
            // Only allow letters, punctuation, and spaces.
            if c.is_alphabetic() || c.is_ascii_punctuation() || c == ' ' {
                naming_input.push(c);
            }
        }
        // Handle backspace to delete the last character.
        if is_key_pressed(KeyCode::Backspace) {
            naming_input.pop();
        }
        // If Enter is pressed and the name isn't empty, finalize the name.
        if is_key_pressed(KeyCode::Enter) && !naming_input.trim().is_empty() {
            my_pet.name = naming_input.trim().to_string();
            naming_done = true;
        }
        // Yield execution back to the browser/engine for one frame.
        next_frame().await;
    }

    // Initialize the game objects after the naming phase.
    let mut ball = Ball::new();
    
        // Bugs that chase the pet.
        // Initialize them at a safe distance from the origin (where the pet starts).
        let mut bugs = vec![
            Bug::new(30.0, 30.0),
            Bug::new(-30.0, 45.0),
            Bug::new(45.0, -30.0),
        ];
    // Speed of the bug.
    let mut current_bug_speed: f32 = 0.12;
    // Timers for spawning and speed increases.
    let mut last_bug_spawn = get_time();
    let mut last_speed_increase = get_time();

    // Default system message displayed in the HUD.
    let mut message = "Take care of your 3D pet!".to_string();
    // Start the game in ThirdPerson camera mode.
    let mut camera_mode = CameraMode::ThirdPerson;

    // Main game loop: runs every frame while the application is open.
    loop {
        // Clear the background to start a fresh frame with a sky-blue color.
        clear_background(SKYBLUE);
        
        // Toggle camera mode when the 'V' key is pressed.
        if is_key_pressed(KeyCode::V) {
            camera_mode = match camera_mode {
                CameraMode::FirstPerson => CameraMode::ThirdPerson,
                CameraMode::ThirdPerson => CameraMode::FirstPerson,
            };
            // Update the HUD message to inform the user of the switch.
            message = format!("Switched to {:?} POV", camera_mode);
        }

        // Update the pet's logic (movement, stats, etc.).
        my_pet.update(matches!(camera_mode, CameraMode::FirstPerson));
        // Update the ball's logic (physics, rotation).
        ball.update();

        // Update bugs and handle interactions.
        for bug in bugs.iter_mut() {
            // Ensure the bug uses the current global speed.
            bug.speed = current_bug_speed;
            bug.update(my_pet.x, my_pet.z);
            
            // Check for collision between bug and pet.
            let bdx = my_pet.x - bug.x;
            let bdz = my_pet.z - bug.z;
            let bdist = (bdx * bdx + bdz * bdz).sqrt();
            if bdist < 1.0 && my_pet.is_alive {
                // If a bug hits the pet, the pet is killed instantly!
                my_pet.is_alive = false;
                // Record the exact time of death to freeze the survival timer.
                if my_pet.death_time.is_none() {
                    my_pet.death_time = Some(get_time());
                }
                message = format!("{} was killed by a bug!", my_pet.name);
            } else if bdist < 2.0 && my_pet.is_alive {
                // If a bug is just very close, it still drains stats and warns the player.
                my_pet.happiness -= 0.1;
                my_pet.energy -= 0.05;
                message = "A bug is closing in!".to_string();
            }
        }

        // Spawn 67 new bugs every second (DEMON DIFFICULTY).
        if get_time() - last_bug_spawn > 1.0 {
            for _ in 0..67 {
                // Spawn bugs at random angles around the pet.
                // Using a slightly more random angle than before.
                let rand_angle = (rand::gen_range(0, 360) as f32).to_radians();
                // Ensure bugs spawn far enough away to avoid "instant spawning on you".
                // Distance increases as the bugs get faster, providing a reaction window.
                let dist = 50.0 + (current_bug_speed * 2.0).min(500.0);
                bugs.push(Bug::new(my_pet.x + rand_angle.cos() * dist, my_pet.z + rand_angle.sin() * dist));
            }
            last_bug_spawn = get_time();
        }

        // Increase bug speed by a very small amount every 5 seconds (DEMON DIFFICULTY).
        if get_time() - last_speed_increase > 5.0 {
            current_bug_speed += 0.02; // Very small speed increase.
            last_speed_increase = get_time();
            message = "Speed slightly increased...".to_string();
        }

        // Interaction Logic: Pet and Ball collision detection.
        let dx = my_pet.x - ball.x;
        let dz = my_pet.z - ball.z;
        // Calculate the horizontal distance between the pet and the ball.
        let dist = (dx * dx + dz * dz).sqrt();
        // If they are close enough (collision radius), move the ball away.
        if dist < 1.5 {
            // Push the ball in the opposite direction of the pet.
            ball.vx = -dx * 0.2;
            ball.vz = -dz * 0.2;
            // Interacting with the ball increases the pet's happiness.
            my_pet.happiness += 1.0;
        }

        // Configure the 3D camera based on the current camera mode.
        match camera_mode {
            CameraMode::ThirdPerson => {
                // Third Person: Camera follows the pet from behind.
                set_camera(&Camera3D {
                    // Position is behind the pet based on its yaw and 10 units away, at a height of 6.
                    position: vec3(my_pet.x - my_pet.yaw.sin() * 10.0, 6.0, my_pet.z - my_pet.yaw.cos() * 10.0),
                    // "Up" vector defines which way is up in world space.
                    up: vec3(0.0, 1.0, 0.0),
                    // The camera looks towards the pet's position.
                    target: vec3(my_pet.x, 1.0, my_pet.z),
                    ..Default::default()
                });
            }
            CameraMode::FirstPerson => {
                // First Person: Camera is inside/at the head of the pet.
                // Calculate the direction the pet is looking using spherical coordinates.
                let look_dir = vec3(
                    my_pet.yaw.sin() * my_pet.pitch.cos(),
                    my_pet.pitch.sin(),
                    my_pet.yaw.cos() * my_pet.pitch.cos()
                );
                // Position the "head" with a slight vertical bobbing effect using a sine wave.
                // Shifted forward and slightly higher to prevent being "inside" the body.
                let head_pos = vec3(my_pet.x, 1.4 + (get_time().sin() * 0.1) as f32, my_pet.z) + look_dir * 1.2;
                set_camera(&Camera3D {
                    position: head_pos,
                    up: vec3(0.0, 1.0, 0.0),
                    // The target is just ahead of the camera in the look direction.
                    target: head_pos + look_dir,
                    ..Default::default()
                });
            }
        }

        // Draw the Sun: A bright yellow sphere in the distance.
        // It's placed far away so it feels like it's in the sky.
        draw_sphere(vec3(50.0, 100.0, 50.0), 10.0, None, YELLOW);
        
        // Procedural Infinite Terrain Generation (Chunk-based rendering).
        let chunk_size = 20.0;
        // Render distance: number of chunks to draw in each direction around the player.
        let view_dist = 2; 
        // Determine which chunk the pet is currently standing in.
        let p_chunk_x = (my_pet.x / chunk_size).floor() as i32;
        let p_chunk_z = (my_pet.z / chunk_size).floor() as i32;

        // Iterate through all chunks within the render distance.
        for cx in (p_chunk_x - view_dist)..=(p_chunk_x + view_dist) {
            for cz in (p_chunk_z - view_dist)..=(p_chunk_z + view_dist) {
                // Calculate world coordinates for the corner of the chunk.
                let x = cx as f32 * chunk_size;
                let z = cz as f32 * chunk_size;
                
                // Deterministic Randomization: generate a unique "seed" for this specific chunk.
                // This ensures that the same chunk always looks the same when the player returns to it.
                let seed = (cx * 73856093 ^ cz * 19349663) as u32;
                let mut rng = seed;
                // A simple Linear Congruential Generator (LCG) for randomness.
                let mut next_rng = || {
                    rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                    (rng >> 16) & 0x7fff
                };

                // Draw the ground plane for this chunk as a large cube.
                // Alternate colors to create a checkerboard pattern for better visual depth.
                let ground_color = if (cx + cz) % 2 == 0 { DARKGREEN } else { GREEN };
                draw_cube(vec3(x + chunk_size/2.0, -0.5, z + chunk_size/2.0), vec3(chunk_size, 1.0, chunk_size), None, ground_color);
                
                // Add "Minecraft-like" environmental features (trees and flowers) to the chunk.
                let num_features = (next_rng() % 5) + 2; // Increased number of features for a more detailed world.
                for _ in 0..num_features {
                    // Randomly position the feature within the chunk.
                    let fx = x + (next_rng() % 100) as f32 / 100.0 * chunk_size;
                    let fz = z + (next_rng() % 100) as f32 / 100.0 * chunk_size;
                    // Randomly decide which type of feature to draw.
                    let f_type = next_rng() % 4;
                    match f_type {
                        0 => {
                            // Grassy hill: a large green sphere on the ground.
                            draw_sphere(vec3(fx, 0.2, fz), 1.5, None, LIME);
                        }
                        1 => {
                            // Tree: both trunk and leaves are placed together at the same (fx, fz).
                            // Trunk: a brown cube at the base.
                            draw_cube(vec3(fx, 1.0, fz), vec3(0.5, 2.0, 0.5), None, BROWN);
                            // Leaves: a green sphere on top of the trunk.
                            draw_sphere(vec3(fx, 2.0, fz), 1.2, None, GREEN);
                        }
                        2 => {
                            // Flower: a small red sphere on a stem.
                            draw_cube(vec3(fx, 0.2, fz), vec3(0.1, 0.5, 0.1), None, GREEN);
                            draw_sphere(vec3(fx, 0.5, fz), 0.2, None, RED);
                        }
                        _ => {
                            // Blue Flower / Berry Bush: a small blue sphere.
                            draw_sphere(vec3(fx, 0.3, fz), 0.3, None, BLUE);
                        }
                    }
                }
            }
        }

        // Draw game entities if the pet is still alive.
        if my_pet.is_alive {
            // Calculate the 3D position of the pet with bobbing animation.
            let pet_pos = vec3(my_pet.x, 1.0 + (get_time().sin() * 0.2) as f32, my_pet.z);
            
            // Calculate the direction vectors for the face and sides based on yaw.
            let face_dir_x = my_pet.yaw.sin();
            let face_dir_z = my_pet.yaw.cos();
            let side_x = my_pet.yaw.cos();
            let side_z = -my_pet.yaw.sin();
            
            // Rolling Animation: Calculate the position of "spots" on the pet's body.
            // These rotate vertically based on rotation_v.
            let roll_sin = my_pet.rotation_v.sin() * 0.8;
            let roll_cos = my_pet.rotation_v.cos() * 0.8;
            
            // Draw the main body of the pet (a sphere).
            draw_sphere(pet_pos, 1.0, None, ORANGE);

            
            // Draw the rolling spots (yellow and brown) to visualize movement.
            let spot1 = pet_pos + vec3(face_dir_x * roll_cos, roll_sin, face_dir_z * roll_cos);
            let spot2 = pet_pos - vec3(face_dir_x * roll_cos, roll_sin, face_dir_z * roll_cos);
            draw_sphere(spot1, 0.2, None, YELLOW);
            draw_sphere(spot2, 0.2, None, BROWN);

            // Positioning the Eyes relative to the body and rotation.
            let eye_offset_y = 0.3;
            let eye_dist = 0.8; // Distance from center to front.
            let eye_spacing = 0.4; // Distance between eyes.
            
            // Left eye position.
            let eye_l = pet_pos + vec3(face_dir_x * eye_dist + side_x * eye_spacing, eye_offset_y, face_dir_z * eye_dist + side_z * eye_spacing);
            // Right eye position.
            let eye_r = pet_pos + vec3(face_dir_x * eye_dist - side_x * eye_spacing, eye_offset_y, face_dir_z * eye_dist - side_z * eye_spacing);
            
            // Draw the eyes (black spheres).
            draw_sphere(eye_l, 0.15, None, BLACK);
            draw_sphere(eye_r, 0.15, None, BLACK);
            
            // Draw the Mouth (a black rectangle/cube).
            let mouth_pos = pet_pos + vec3(face_dir_x * eye_dist, -0.3, face_dir_z * eye_dist);
            draw_cube(mouth_pos, vec3(0.4, 0.1, 0.1), None, BLACK);

            // Draw the Bugs.
            for bug in &bugs {
                // Bugs are small black spheres that hover slightly above the ground.
                let bug_pos = vec3(bug.x, 0.5 + (get_time() * 5.0).sin() as f32 * 0.1, bug.z);
                draw_sphere(bug_pos, 0.3, None, BLACK);
                // Draw little bug eyes (red).
                draw_sphere(bug_pos + vec3(0.1, 0.1, 0.2), 0.05, None, RED);
                draw_sphere(bug_pos + vec3(-0.1, 0.1, 0.2), 0.05, None, RED);
            }

            // Render the Ball.
            let ball_pos = vec3(ball.x, 0.5, ball.z);
            // Draw the ball body (white sphere).
            draw_sphere(ball_pos, 0.5, None, WHITE);
            // Draw two colored spots on the ball that move based on its rotation field.
            let rot_x = ball.rotation.cos() * 0.4;
            let rot_y = ball.rotation.sin() * 0.4;
            draw_sphere(ball_pos + vec3(rot_x, rot_y, 0.3), 0.1, None, RED);
            draw_sphere(ball_pos + vec3(-rot_x, -rot_y, -0.3), 0.1, None, BLUE);
        }

        // Switch the rendering context back to 2D to draw the User Interface (HUD).
        set_default_camera();

        // Draw the Heads-Up Display (HUD).
        // Display the pet's name.
        draw_text(&format!("Name: {}", my_pet.name), 20.0, 30.0, 30.0, BLACK);
        
        // Calculate the current survival time if the pet is alive.
        if my_pet.is_alive {
            let survival_time = get_time() - my_pet.start_time;
            draw_text(&format!("Survived: {:.1}s", survival_time), 20.0, 50.0, 20.0, DARKGRAY);
            draw_text("DIFFICULTY: 67 BUGS/S", 20.0, 70.0, 20.0, RED);
        }

        // Draw the status bars for Hunger, Happiness, and Energy.
        draw_bar(80.0, "Hunger", my_pet.hunger, RED);
        draw_bar(110.0, "Happiness", my_pet.happiness, GREEN);
        draw_bar(140.0, "Energy", my_pet.energy, BLUE);

        // Check if the pet has died and display the Game Over screen.
        if !my_pet.is_alive {
            // Use the frozen death time if available.
            let final_survival = my_pet.death_time.unwrap_or(get_time()) - my_pet.start_time;
            draw_text("GAME OVER", screen_width() / 2.0 - 100.0, screen_height() / 2.0 - 40.0, 50.0, RED);
            draw_text(&format!("You survived for {:.2} seconds!", final_survival), screen_width() / 2.0 - 120.0, screen_height() / 2.0 + 10.0, 25.0, BLACK);
            draw_text("Press R to restart", screen_width() / 2.0 - 80.0, screen_height() / 2.0 + 50.0, 20.0, DARKGRAY);
            
            // Restart logic: allow the user to rename the pet and start over.
            if is_key_pressed(KeyCode::R) {
                naming_input.clear();
                naming_done = false;
                // Nested loop for the renaming screen upon restart.
                while !naming_done {
                    clear_background(LIGHTGRAY);
                    draw_text("Rename your 3D pet:", 20.0, 100.0, 30.0, BLACK);
                    draw_text(&naming_input, 20.0, 150.0, 40.0, BLUE);
                    draw_text("Press ENTER to start", 20.0, 200.0, 20.0, DARKGRAY);
                    // Same input logic as the initial naming screen.
                    if let Some(c) = get_char_pressed() {
                        if c.is_alphabetic() || c.is_ascii_punctuation() || c == ' ' {
                            naming_input.push(c);
                        }
                    }
                    if is_key_pressed(KeyCode::Backspace) {
                        naming_input.pop();
                    }
                    if is_key_pressed(KeyCode::Enter) && !naming_input.trim().is_empty() {
                        // Create a completely new Pet instance with the new name.
                        my_pet = Pet::new(naming_input.trim().to_string());
                        naming_done = true;
                    }
                    next_frame().await;
                }
                // Reset the ball and the game message.
                ball = Ball::new();
                // Reset bugs and speed on restart.
                // Initialize at a safe distance from the pet's current location.
                bugs = vec![
                    Bug::new(my_pet.x + 30.0, my_pet.z + 30.0),
                    Bug::new(my_pet.x - 30.0, my_pet.z + 45.0),
                    Bug::new(my_pet.x + 45.0, my_pet.z - 30.0),
                ];
                current_bug_speed = 0.12;
                last_bug_spawn = get_time();
                last_speed_increase = get_time();
                message = "Welcome back!".to_string();
            }
        } else {
            // If the pet is alive, display the system message and controls.
            draw_text(&message, 20.0, screen_height() - 60.0, 25.0, DARKGRAY);
            draw_text("1: Feed | 2: Play | 3: Sleep | WASD: Move/Turn | V: POV", 20.0, screen_height() - 30.0, 20.0, BLACK);

            // Handle manual interaction keys (1, 2, 3).
            if is_key_pressed(KeyCode::Key1) {
                if my_pet.feed() {
                    message = format!("You fed {}!", my_pet.name);
                } else {
                    message = "Stand still to eat!".to_string();
                }
            }
            if is_key_pressed(KeyCode::Key2) {
                if my_pet.play() {
                    message = format!("You played with {}!", my_pet.name);
                } else {
                    message = "Stand still to play!".to_string();
                }
            }
            if is_key_pressed(KeyCode::Key3) {
                if my_pet.sleep() {
                    message = format!("{} is sleeping...", my_pet.name);
                } else {
                    message = "Stand still to sleep!".to_string();
                }
            }
        }

        // CRITICAL: Flush the character pressed buffer at the end of every frame.
        // This prevents movement keys (W, A, S, D) from being interpreted as text input
        // if the naming screen is triggered immediately after movement.
        while get_char_pressed().is_some() {}

        // End of frame: wait for the next vertical sync.
        next_frame().await
    }
}
