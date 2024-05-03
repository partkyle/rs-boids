use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::window::WindowResolution;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_egui::egui::lerp;
use rand::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, (setup_camera, setup))
        .add_systems(Update, (boids_ui, update_boid_velocity, update_boid_direction, boid_turn_factor))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct BoidVisualData {
    shape: Handle<Mesh>,
    color: Handle<ColorMaterial>,
}

fn setup(mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let size = 10.0;
    let shape = meshes.add(Triangle2d::new(
        Vec2::Y * size,
        Vec2::new(-size / 2.0, -size),
        Vec2::new(size / 2.0, -size),
    ));

    let color = materials.add(Color::rgb(0.0, 1.0, 1.0));

    commands.spawn_empty().insert(BoidVisualData { shape, color });

    commands.spawn_empty().insert(BoidConfiguration::default());
}


#[derive(Component, Debug)]
struct BoidConfiguration {
    spawn_count: u32,
    spawn_range: Rect,
    turn_factor: f32,
    visual_range: f32,
    protected_range: f32,
    avoid_factor: f32,
    centering_factor: f32,
    matching_factor: f32,
    max_speed: f32,
    min_speed: f32,
}

impl Default for BoidConfiguration {
    fn default() -> Self {
        BoidConfiguration {
            spawn_count: 100,
            spawn_range: Rect { min: Vec2::new(-200.0, -200.0), max: Vec2::new(200.0, 200.0) },
            turn_factor: 0.2,
            visual_range: 20.0,
            protected_range: 2.0,
            centering_factor: 0.0005,
            avoid_factor: 0.05,
            matching_factor: 0.05,
            max_speed: 100.0,
            min_speed: 2.0,
        }
    }
}

fn boids_ui(
    mut commands: Commands,
    mut config: Query<&mut BoidConfiguration>,
    mut contexts: EguiContexts,
    bvd: Query<&BoidVisualData>,
) {
    let mut config = config.single_mut();
    let bvd = bvd.single();

    bevy_egui::egui::Window::new("boids").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.spawn_count, 1..=10000u32),
            );

            if ui.button("spawn").clicked() {
                for i in 0..config.spawn_count {
                    spawn_boid(&mut commands, bvd, &config);
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("turn_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.turn_factor, 0.0..=10.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("visual_range");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.visual_range, 0.0..=10.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("protected_range");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.protected_range, 0.0..=10.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("centering_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.centering_factor, 0.0..=10.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("avoid_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.avoid_factor, 0.0..=10.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("matching_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.matching_factor, 0.0..=10.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("max_speed");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.max_speed, 0.0..=100.0f32),
            );
        });

        ui.horizontal(|ui| {
            ui.label("min_speed");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.min_speed, 0.0..=100.0f32),
            );
        });
    });
}


#[derive(Component, Default)]
struct Boid {
    velocity: Vec2,
}

fn spawn_boid(commands: &mut Commands, bvd: &BoidVisualData, config: &BoidConfiguration) {
    commands
        .spawn_empty().
        insert(Boid {
            velocity: Vec2 { x: lerp(-config.max_speed..=config.max_speed, random::<f32>()), y: lerp(-config.max_speed..=config.max_speed, random::<f32>()) },
            ..default()
        })
        .insert(ColorMesh2dBundle {
            mesh: Mesh2dHandle(bvd.shape.clone()),
            material: bvd.color.clone(),
            transform: Transform::from_xyz(lerp(config.spawn_range.min.x..=config.spawn_range.max.x, random::<f32>()), lerp(config.spawn_range.min.y..=config.spawn_range.max.y, random::<f32>()), 0.0),
            ..default()
        });
}

fn update_boid_velocity(time: Res<Time>, mut boids: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in boids.iter_mut() {
        let new_pos = transform.translation.xy() + boid.velocity * time.delta().as_secs_f32();
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn update_boid_direction(time: Res<Time>, mut boids: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in boids.iter_mut() {
        let angle = boid.velocity.x.atan2(boid.velocity.y);
        transform.rotation = Quat::from_axis_angle(Vec3::NEG_Z, angle);
    }
}

fn boid_turn_factor(time: Res<Time>, window: Query<&Window>, config: Query<&BoidConfiguration>, mut boids: Query<(&mut Boid, &Transform)>) {
    let window = window.single();
    let config = config.single();
    let w = window.width() / 2.0;
    let h = window.height() / 2.0;
    for (mut boid, transform) in boids.iter_mut() {
        if transform.translation.x < -w {
            boid.velocity.x += config.turn_factor;
        }

        if transform.translation.x > w {
            boid.velocity.x -= config.turn_factor;
        }

        if transform.translation.y < h {
            boid.velocity.y += config.turn_factor;
        }

        if transform.translation.y > -h {
            boid.velocity.y -= config.turn_factor;
        }
    }
}