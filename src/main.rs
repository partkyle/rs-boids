use bevy::{
    prelude::*,
    sprite::Mesh2dHandle,
};
use bevy::render::mesh::PrimitiveTopology::LineList;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::window::close_on_esc;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_egui::egui::lerp;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use rand::random;

use crate::quadtree::Quadtree;

pub mod quadtree;

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
        .add_plugins(ShapePlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (setup_camera, setup))
        .add_systems(Update, (close_on_esc, populate_quadtree, render_quadtree, boids_ui, boid_separation, boid_speed_up, boid_rotation, boid_turn_factor, boid_movement))
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
         window: Query<&Window>,
) {
    let window = window.single();
    let size = 10.0;
    let shape = meshes.add(Triangle2d::new(
        Vec2::Y * size,
        Vec2::new(-size / 2.0, -size),
        Vec2::new(size / 2.0, -size),
    ));

    let color = materials.add(Color::rgb(0.0, 1.0, 1.0));

    commands.spawn_empty().insert(BoidVisualData { shape, color });


    let config = BoidConfiguration {
        boid_bounds: Rect::new(-window.width() / 2.0, -window.height() / 2.0, window.width() / 2.0, window.height() / 2.0),
        ..default()
    };

    println!("{:?}", config.boid_bounds);

    let color = materials.add(Color::rgb(0.0, 1.0, 0.0));
    let tree_jail = TreeJail::new(config.boid_bounds, 100);
    commands.spawn_empty()
        .insert(tree_jail)
        .insert(SpatialBundle::default())
        .insert(color);

    commands.spawn_empty().insert(config);
}


#[derive(Component, Debug)]
struct BoidConfiguration {
    spawn_count: u32,
    spawn_range: Rect,
    turn_factor: f32,
    boid_bounds: Rect,
    visual_range: f32,
    protected_range: f32,
    avoid_factor: f32,
    centering_factor: f32,
    matching_factor: f32,
    max_speed: f32,
    min_speed: f32,

    render_quadtree: bool,
}

impl Default for BoidConfiguration {
    fn default() -> Self {
        BoidConfiguration {
            spawn_count: 100,
            spawn_range: Rect { min: Vec2::new(-200.0, -200.0), max: Vec2::new(200.0, 200.0) },
            turn_factor: 0.2,
            boid_bounds: Rect { min: Vec2::new(-200.0, -200.0), max: Vec2::new(200.0, 200.0) },
            visual_range: 20.0,
            protected_range: 2.0,
            centering_factor: 0.0005,
            avoid_factor: 0.05,
            matching_factor: 0.05,
            max_speed: 100.0,
            min_speed: 2.0,

            render_quadtree: false,
        }
    }
}

fn boids_ui(
    mut commands: Commands,
    mut config: Query<&mut BoidConfiguration>,
    mut contexts: EguiContexts,
    boids: Query<Entity, With<Boid>>,
    bvd: Query<&BoidVisualData>,
) {
    let mut config = config.single_mut();
    let bvd = bvd.single();

    egui::Window::new("boids").show(contexts.ctx_mut(), |ui| {
        egui::Grid::new("something").show(ui, |ui| {
            ui.label("spawn count");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.spawn_count, 1..=10000u32),
            );

            if ui.button("spawn").clicked() {
                for _ in 0..config.spawn_count {
                    spawn_boid(&mut commands, bvd, &config);
                }
            }
            ui.end_row();

            if ui.button("despawn").clicked() {
                for entity in boids.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
            ui.end_row();

            ui.label("turn_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.turn_factor, 0.0..=10.0f32),
            );
            ui.end_row();

            ui.label("visual_range");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.visual_range, 0.0..=10.0f32),
            );
            ui.end_row();

            ui.label("protected_range");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.protected_range, 0.0..=1000.0f32),
            );
            ui.end_row();

            ui.label("centering_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.centering_factor, 0.0..=10.0f32),
            );
            ui.end_row();

            ui.label("avoid_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.avoid_factor, 0.0..=10.0f32),
            );
            ui.end_row();

            ui.label("matching_factor");
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.matching_factor, 0.0..=10.0f32),
            );
            ui.end_row();

            ui.label("max_speed");
            let min = config.min_speed;
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.max_speed, min..=1000.0f32)
            );
            ui.end_row();

            ui.label("min_speed");
            let max = config.max_speed;
            ui.add(
                bevy_egui::egui::Slider::new(&mut config.min_speed, 0.0..=max),
            );
            ui.end_row();

            ui.checkbox(&mut config.render_quadtree, "render_quadtree");
        });
    });
}


#[derive(Component, Default)]
struct Boid {
    velocity: Vec2,
}

fn spawn_boid(commands: &mut Commands, bvd: &BoidVisualData, config: &BoidConfiguration) {
    let shape = shapes::Circle {
        center: Vec2::ZERO,
        radius: config.protected_range,
        ..default()
    };
    let range_radius = commands.spawn(
        (
            Name::new("protected_range"),
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Stroke::new(Color::RED, 1.0),
        )
    ).id();

    commands.spawn(
        (
            Name::new("boid"),
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(bvd.shape.clone()),
                material: bvd.color.clone(),
                transform: Transform::from_xyz(lerp(config.spawn_range.min.x..=config.spawn_range.max.x, random::<f32>()), lerp(config.spawn_range.min.y..=config.spawn_range.max.y, random::<f32>()), 0.0),
                ..default()
            },
            Boid {
                velocity: Vec2 {
                    x: lerp(-config.max_speed..=config.max_speed, random::<f32>()),
                    y: lerp(-config.max_speed..=config.max_speed, random::<f32>()),
                },
                ..default()
            },
        )
    ).add_child(range_radius);
}

fn boid_movement(time: Res<Time>, mut boids: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in boids.iter_mut() {
        let new_pos = transform.translation.xy() + boid.velocity * time.delta().as_secs_f32();
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn boid_rotation(mut boids: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in boids.iter_mut() {
        let angle = boid.velocity.x.atan2(boid.velocity.y);
        transform.rotation = Quat::from_axis_angle(Vec3::NEG_Z, angle);
    }
}

fn boid_separation(mut boids: Query<(Entity, &mut Boid, &mut Transform)>, tree_jail: Query<&TreeJail>, config: Query<&BoidConfiguration>) {
    let config = config.single();
    let tree_jail = tree_jail.single();
    for (entity, mut boid, mut transform) in boids.iter_mut() {
        // tree_jail.quadtree
        let position = transform.translation.xy();
        let min = position - (config.protected_range / 2.0);
        let max = position + (config.protected_range / 2.0);
        let mut results = vec![];
        tree_jail.quadtree.query(&Rect { min, max }, &mut results);

        let mut dclose = Vec2::ZERO;
        let mut count = 0;
        let total = results.len();
        for (other_position, other_entity) in results {
            if entity == other_entity {
                continue;
            }

            let distance = position - other_position;
            if distance.length() <= config.protected_range {
                dclose += distance;
                count += 1;
            }
        }

        boid.velocity += dclose * config.avoid_factor;
    }
}

fn boid_speed_up(time: Res<Time>, mut boids: Query<&mut Boid>, config: Query<&BoidConfiguration>) {
    let config = config.single();
    for mut boid in boids.iter_mut() {
        let before = boid.velocity;
        if boid.velocity.length() <= config.max_speed {
            boid.velocity = boid.velocity.lerp(boid.velocity.normalize() * config.max_speed, time.delta_seconds());
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

#[derive(Component)]
struct TreeJail {
    quadtree: Quadtree<Entity>,
}

impl TreeJail {
    fn new(bounds: Rect, capacity: usize) -> TreeJail {
        TreeJail {
            quadtree: quadtree::Quadtree::new(bounds, capacity),
        }
    }
}

fn populate_quadtree(mut commands: Commands, config: Query<&BoidConfiguration>, mut tree_jail: Query<&mut TreeJail>, boids: Query<(Entity, &Transform), With<Boid>>) {
    let config = config.single();
    let mut tree_jail = tree_jail.single_mut();
    tree_jail.quadtree = quadtree::Quadtree::new(Rect { min: config.boid_bounds.min * 1.5, max: config.boid_bounds.max * 1.5 }, 1);
    for (entity, transform) in boids.iter() {
        tree_jail.quadtree.insert(transform.translation.xy(), entity);
    }
}

fn render_quadtree(mut commands: Commands, config: Query<&BoidConfiguration>, tree_jail: Query<(Entity, &TreeJail, &Handle<ColorMaterial>)>, shapes: Query<(Entity, &Path)>) {
    let config = config.single();

    let (entity, tree_jail, color) = tree_jail.single();

    commands.entity(entity).despawn_descendants();

    if !config.render_quadtree {
        return;
    }

    let mut children = vec![];
    for b in tree_jail.quadtree.get_all_bounds() {
        let size = b.max - b.min;
        let shape = shapes::Rectangle {
            extents: size,
            origin: RectangleOrigin::BottomLeft,
            ..default()
        };
        children.push(
            commands.spawn_empty()
            .insert(
                (
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        spatial: SpatialBundle {
                            transform: Transform::from_xyz(b.min.x, b.min.y, -1.0),
                            ..default()
                        },
                        ..default()
                    },
                    Stroke::new(Color::GREEN, 1.0),
                )
            ).id()
        );
    }

    commands.entity(entity).replace_children(&children);
}