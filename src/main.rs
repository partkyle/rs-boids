use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::window::close_on_esc;
use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_egui::egui::lerp;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::random;

mod config;
mod environ;
mod quadtree;
mod quadtree_gizmos;
mod range_gizmos;

use config::{BoidConfiguration, ColorType};
use environ::default_plugins;
use quadtree::Quadtree;
use quadtree_gizmos::render_quadtree;
use range_gizmos::boid_draw_range_gizmos;

#[derive(Resource, Deref, DerefMut)]
struct QuadtreeJail(Quadtree<EntityWrapper>);

fn main() {
    App::new()
        .add_plugins(default_plugins())
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(QuadtreeJail(quadtree::Quadtree::new(
            Rect::new(-10000.0, -10000.0, 10000.0, 10000.0),
            1,
        )))
        .add_systems(Startup, (setup_camera, setup, spawn_1000).chain())
        .add_systems(
            Update,
            (
                close_on_esc,
                render_quadtree,
                boids_ui,
                boid_draw_range_gizmos,
                render_bounds_gizmo,
                boid_rotation,
                boid_update_colors,
                update_boids_transform,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                populate_quadtree,
                boid_flocking_behaviors,
                boid_turn_factor,
                boid_speed_up,
                boid_movement,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct BoidVisualData {
    shape: Handle<Mesh>,
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, window: Query<&Window>) {
    let window = window.single();
    let size = 10.0;
    let shape = meshes.add(Triangle2d::new(
        Vec2::Y * size,
        Vec2::new(-size / 2.0, -size),
        Vec2::new(size / 2.0, -size),
    ));

    commands.spawn_empty().insert(BoidVisualData { shape });

    let config = BoidConfiguration {
        boid_bounds: Rect::new(
            -window.width() / 2.0,
            -window.height() / 2.0,
            window.width() / 2.0,
            window.height() / 2.0,
        ),
        ..default()
    };

    commands.spawn_empty().insert(config);
}

fn boids_ui(
    mut commands: Commands,
    mut config: Query<&mut BoidConfiguration>,
    mut contexts: EguiContexts,
    mut materials: ResMut<Assets<ColorMaterial>>,
    boids: Query<Entity, With<Boid>>,
    bvd: Query<&BoidVisualData>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut config = config.single_mut();
    let bvd = bvd.single();

    egui::Window::new("boids").show(contexts.ctx_mut(), |ui| {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            egui::Grid::new("fps").show(ui, |ui| {
                ui.label("fps");
                ui.label(format!("{:.2}", fps));
            });
        }

        ui.heading("Spawning Fields");
        egui::Grid::new("spawn_fields").show(ui, |ui| {
            ui.label("total boids");
            ui.label(config.total_boids.to_string());
            ui.end_row();

            ui.label("spawn count");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.spawn_count,
                1..=10000u32,
            ));

            if ui.button("spawn").clicked() {
                for _ in 0..config.spawn_count {
                    spawn_boid(&mut commands, bvd, &mut config, &mut materials);
                }
            }

            if ui.button("despawn").clicked() {
                for entity in boids.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                config.total_boids = 0;
            }
            ui.end_row();
        });

        ui.heading("Simulation Fields");
        egui::Grid::new("simulation_fields").show(ui, |ui| {
            ui.label("turn_factor");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.turn_factor,
                0.0..=10.0f32,
            ));
            ui.end_row();

            ui.label("visible_range");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.visible_range,
                0.0..=100.0f32,
            ));
            ui.end_row();

            ui.label("protected_range");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.protected_range,
                0.0..=100.0f32,
            ));
            ui.end_row();

            ui.label("centering_factor");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.centering_factor,
                0.0..=10.0f32,
            ));
            ui.end_row();

            ui.label("avoid_factor");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.avoid_factor,
                0.0..=10.0f32,
            ));
            ui.end_row();

            ui.label("matching_factor");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.matching_factor,
                0.0..=10.0f32,
            ));
            ui.end_row();

            ui.label("max_speed");
            let min = config.min_speed;
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.max_speed,
                min..=1000.0f32,
            ));
            ui.end_row();

            ui.label("min_speed");
            let max = config.max_speed;
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.min_speed,
                0.0..=max,
            ));
            ui.end_row();
        });

        ui.heading("Gizmos");
        ui.checkbox(&mut config.render_bounds, "render_bounds");
        ui.checkbox(&mut config.render_quadtree, "render_quadtree");
        ui.checkbox(&mut config.render_protected_range, "render_protected_range");
        ui.checkbox(&mut config.render_visible_range, "render_visible_range");

        ui.heading("Boid Colors");
        ui.horizontal(|ui| {
            ui.label("update_color_sample_rate");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.update_color_sample_rate,
                0.0..=1.0f32,
            ));
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut config.update_color_type, ColorType::Initial, "Initial");
            ui.radio_value(
                &mut config.update_color_type,
                ColorType::Synthwave,
                "Synthwave",
            );
            ui.radio_value(&mut config.update_color_type, ColorType::Pastel, "Pastel");
            ui.radio_value(
                &mut config.update_color_type,
                ColorType::PrimaryRGB,
                "PrimaryRGB",
            );
        });
    });
}

#[derive(Component, Default)]
struct Boid {
    position: Vec2,
    velocity: Vec2,
    initial_color: Color,
}

fn spawn_1000(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config: Query<&mut BoidConfiguration>,
    bvd: Query<&BoidVisualData>,
) {
    let mut config = config.single_mut();
    let bvd = bvd.single();
    for _ in 0..1000 {
        spawn_boid(&mut commands, bvd, &mut config, &mut materials)
    }
}

fn spawn_boid(
    commands: &mut Commands,
    bvd: &BoidVisualData,
    config: &mut BoidConfiguration,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let entity = commands.spawn_empty().id();

    let initial_color = Color::rgb(random(), random(), random());

    let position = Vec2::new(
        lerp(
            config.spawn_range.min.x..=config.spawn_range.max.x,
            random::<f32>(),
        ),
        lerp(
            config.spawn_range.min.y..=config.spawn_range.max.y,
            random::<f32>(),
        ),
    );

    commands.entity(entity).insert((
        Name::new("boid"),
        ColorMesh2dBundle {
            mesh: Mesh2dHandle(bvd.shape.clone()),
            // material: materials.add(Color::rgb(random(), random(), random())),
            material: materials.add(initial_color),
            transform: Transform::from_xyz(
                position.x,
                position.y,
                // use the entity index for the z value to prevent (war) z-fighting
                entity.index() as f32 * 0.001,
            ),
            ..default()
        },
        Boid {
            position,
            velocity: Vec2 {
                x: lerp(-config.max_speed..=config.max_speed, random::<f32>()),
                y: lerp(-config.max_speed..=config.max_speed, random::<f32>()),
            },
            initial_color,
        },
    ));

    config.total_boids += 1;
}

fn boid_movement(time: Res<Time>, mut boids: Query<&mut Boid>) {
    for mut boid in boids.iter_mut() {
        boid.position = boid.position + boid.velocity * time.delta().as_secs_f32();
    }
}

fn boid_rotation(mut boids: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in boids.iter_mut() {
        let angle = boid.velocity.x.atan2(boid.velocity.y);
        transform.rotation = Quat::from_axis_angle(Vec3::NEG_Z, angle);
    }
}

fn boid_flocking_behaviors(
    mut boids: Query<(Entity, &mut Boid)>,
    qt: Res<QuadtreeJail>,
    config: Query<&BoidConfiguration>,
) {
    let config = config.single();
    for (entity, mut boid) in boids.iter_mut() {
        // tree_jail.quadtree
        let position = boid.position;
        let max_range = config.protected_range.max(config.visible_range);
        let min = position - (max_range / 2.0);
        let max = position + (max_range / 2.0);

        let mut results = vec![];
        qt.query(&Rect { min, max }, &mut results);

        let mut dclose = Vec2::ZERO;

        let mut boids_in_visible_range = 0;
        let mut velocity_avg = Vec2::ZERO;
        let mut position_avg = Vec2::ZERO;
        for (other_position, other_entity) in results {
            if entity == other_entity.entity {
                continue;
            }

            let distance = position - other_position;
            if distance.length() <= config.protected_range {
                dclose += distance;
            }

            if distance.length() <= config.visible_range {
                boids_in_visible_range += 1;
                velocity_avg += other_entity.velocity;

                position_avg += other_position;
            }
        }

        boid.velocity += dclose * config.avoid_factor;

        if boids_in_visible_range > 0 {
            // alignment
            velocity_avg /= boids_in_visible_range as f32;
            boid.velocity = boid.velocity + (velocity_avg - boid.velocity) * config.matching_factor;

            // cohesion
            position_avg /= boids_in_visible_range as f32;
            boid.velocity = boid.velocity + (position_avg - position) * config.centering_factor
        }
    }
}

fn boid_speed_up(time: Res<Time>, mut boids: Query<&mut Boid>, config: Query<&BoidConfiguration>) {
    let config = config.single();
    for mut boid in boids.iter_mut() {
        if boid.velocity.length() <= config.max_speed {
            boid.velocity = boid.velocity.lerp(
                boid.velocity.normalize() * config.max_speed,
                time.delta_seconds(),
            );
        }
    }
}

fn boid_update_colors(
    boids: Query<(&Boid, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Query<&BoidConfiguration>,
) {
    let config = config.single();

    if config.update_color_sample_rate == 0.0 {
        return;
    }

    for (boid, color) in boids.iter() {
        if random::<f32>() <= config.update_color_sample_rate {
            if let Some(color) = materials.get_mut(color.id()) {
                match config.update_color_type {
                    ColorType::Initial => {
                        color.color = boid.initial_color;
                    }
                    ColorType::Synthwave => {
                        let r: f32 = boid.velocity.x.abs() / config.max_speed;
                        let g = boid.velocity.y.abs() / config.max_speed;
                        color.color = Color::rgb(r, g, 1.0);
                    }
                    ColorType::Pastel => {
                        let r: f32 = boid.velocity.x.abs() / config.max_speed;
                        let g = boid.velocity.y.abs() / config.max_speed;
                        color.color = Color::rgb(r, g, (1.0 - r - g).clamp(0.0, 1.0));
                    }
                    ColorType::PrimaryRGB => {
                        let r: f32 = (boid.velocity.x + boid.velocity.x.abs()) / config.max_speed;
                        let g = (boid.velocity.y + boid.velocity.y.abs()) / config.max_speed;
                        color.color = Color::rgb(r, g, (1.0 - r - g).clamp(0.0, 1.0));
                    }
                }
            }
        }
    }
}

fn boid_turn_factor(config: Query<&BoidConfiguration>, mut boids: Query<(&mut Boid, &Transform)>) {
    let config = config.single();
    for (mut boid, transform) in boids.iter_mut() {
        if transform.translation.x < config.boid_bounds.min.x {
            boid.velocity.x += config.turn_factor;
        }

        if transform.translation.x > config.boid_bounds.max.x {
            boid.velocity.x -= config.turn_factor;
        }

        if transform.translation.y < config.boid_bounds.min.y {
            boid.velocity.y += config.turn_factor;
        }

        if transform.translation.y > config.boid_bounds.max.y {
            boid.velocity.y -= config.turn_factor;
        }
    }
}

#[derive(Clone)]
struct EntityWrapper {
    entity: Entity,
    velocity: Vec2,
}

fn populate_quadtree(mut qt: ResMut<QuadtreeJail>, boids: Query<(Entity, &Boid), With<Boid>>) {
    qt.clear();
    for (entity, boid) in boids.iter() {
        qt.insert(
            boid.position,
            EntityWrapper {
                entity,
                velocity: boid.velocity,
            },
        );
    }
}

fn update_boids_transform(mut boids: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in boids.iter_mut() {
        transform.translation.x = boid.position.x;
        transform.translation.y = boid.position.y;
    }
}

pub fn render_bounds_gizmo(config: Query<&BoidConfiguration>, mut gizmos: Gizmos) {
    let config = config.single();

    if !config.render_bounds {
        return;
    }

    let size = config.boid_bounds.max - config.boid_bounds.min;
    let position = config.boid_bounds.min + size * 0.5;
    gizmos.rect_2d(position, 0.0, size, Color::WHITE.with_a(0.1));
}
