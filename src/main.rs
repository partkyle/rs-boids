use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::utils::hashbrown::HashMap;
use bevy::window::{close_on_esc, PrimaryWindow};
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

use config::{BoidConfiguration, BoidGizmoConfig, ColorType};
use environ::default_plugins;
use quadtree::Quadtree;
use quadtree_gizmos::render_quadtree;
use range_gizmos::boid_draw_range_gizmos;

#[derive(Resource, Deref, DerefMut)]
struct QuadtreeJail(Quadtree<EntityWrapper>);

#[derive(States, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
enum SpatialState {
    QuadTree,
    SpatialHash,
}

fn main() {
    App::new()
        .add_plugins(default_plugins())
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_state(SpatialState::SpatialHash)
        .insert_resource(QuadtreeJail(quadtree::Quadtree::new(
            Rect::new(-10000.0, -10000.0, 10000.0, 10000.0),
            1,
        )))
        .add_systems(Startup, (setup_camera, setup, spawn_1000).chain())
        .add_systems(
            Update,
            (
                close_on_esc,
                boids_ui,
                boid_ensure_count.after(boids_ui),
                (
                    render_quadtree,
                    boid_select_randomly,
                    highlight_boid,
                    (populate_quadtree, boid_flocking_behaviors)
                        .run_if(in_state(SpatialState::QuadTree)),
                    (boid_flocking_spatial_hash).run_if(in_state(SpatialState::SpatialHash)),
                    boid_turn_factor,
                    boid_speed_up,
                    boid_movement,
                    boid_draw_range_gizmos,
                    render_bounds_gizmo,
                    boid_rotation,
                    boid_update_colors,
                    boid_highlight_neighbors,
                    update_boids_transform,
                )
                    .after(boid_ensure_count),
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
    mut config: Query<&mut BoidConfiguration>,
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    spatial_state: Res<State<SpatialState>>,
    mut next_spatial_state: ResMut<NextState<SpatialState>>,
) {
    let mut config = config.single_mut();

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

        ui.horizontal(|ui| {
            let mut current = spatial_state.get().clone();
            ui.radio_value(&mut current, SpatialState::QuadTree, "QuadTree");
            ui.radio_value(&mut current, SpatialState::SpatialHash, "SpatialHash");

            if current != *spatial_state.get() {
                next_spatial_state.set(current);
            }
        });

        ui.heading("Spawning Fields");
        egui::Grid::new("spawn_fields").show(ui, |ui| {
            ui.label("boids count");
            ui.add(bevy_egui::egui::Slider::new(
                &mut config.spawn_count,
                1..=10000u32,
            ));
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

        egui::Grid::new("gizmos").show(ui, |ui| {
            ui.heading("Gizmos");
            ui.end_row();

            boid_ui_for_gizmos(ui, "render_bounds", &mut config.bounds_gizmo);
            boid_ui_for_gizmos(ui, "render_quadtree", &mut config.quadtree_gizmo);
            boid_ui_for_gizmos(
                ui,
                "render_protected_range",
                &mut config.protected_range_gizmo,
            );
            boid_ui_for_gizmos(ui, "render_visible_range", &mut config.visible_range_gizmo);
        });

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

fn boid_ui_for_gizmos(ui: &mut bevy_egui::egui::Ui, text: &str, val: &mut BoidGizmoConfig) {
    ui.checkbox(&mut val.enabled, text);
    ui.color_edit_button_rgba_unmultiplied(&mut val.color_rgba);
    ui.end_row();
}

fn boid_ensure_count(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bvd: Query<&BoidVisualData>,
    mut config: Query<&mut BoidConfiguration>,
    boids: Query<Entity, With<Boid>>,
) {
    let mut config = config.single_mut();
    let bvd = bvd.single();

    let current = boids.iter().count() as u32;

    if current < config.spawn_count {
        for _ in 0..(config.spawn_count - current) {
            spawn_boid(&mut commands, bvd, &mut config, &mut materials);
        }
    }

    if current > config.spawn_count {
        let mut to_remove = current - config.spawn_count;
        for entity in boids.iter() {
            commands.entity(entity).despawn_recursive();
            to_remove -= 1;
            if to_remove == 0 {
                break;
            }
        }
    }
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

    let initial_color = Color::rgb(0.0, random(), random());

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
    mut commands: Commands,
    mut boids: Query<(Entity, &mut Boid, Option<&Highlighted>)>,
    qt: Res<QuadtreeJail>,
    config: Query<&BoidConfiguration>,
    old_neighbors: Query<Entity, With<HighlightedNeighbor>>,
) {
    for entity in old_neighbors.iter() {
        commands.entity(entity).remove::<HighlightedNeighbor>();
    }

    let config = config.single();
    for (entity, mut boid, highlighted) in boids.iter_mut() {
        // tree_jail.quadtree
        let position = boid.position;
        let max_range = config.protected_range.max(config.visible_range);
        let min = position - max_range;
        let max = position + max_range;

        let neighbor_boids = qt.query(Rect { min, max });

        let mut dclose = Vec2::ZERO;

        let mut boids_in_visible_range = 0;
        let mut velocity_avg = Vec2::ZERO;
        let mut position_avg = Vec2::ZERO;
        for (other_position, other_entity) in neighbor_boids {
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

                if highlighted.is_some() {
                    commands
                        .entity(other_entity.entity)
                        .insert(HighlightedNeighbor);
                }
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

fn hash_coords(x: u32, y: u32, num_cells: u32) -> u32 {
    let h = (x as u64 * 92837111) ^ (y as u64 * 689287499);
    return (h % num_cells as u64) as u32;
}

fn find_cell_position(position: Vec2, bounds: Rect, cell_size: f32) -> Option<UVec2> {
    let from_bounds = position - bounds.min;
    if from_bounds.x < 0.0
        || from_bounds.y < 0.0
        || from_bounds.x >= bounds.size().x
        || from_bounds.y >= bounds.size().y
    {
        // //skip
        return None;
    }

    let cell_x = (from_bounds.x / cell_size).floor();
    let cell_y = (from_bounds.y / cell_size).floor();

    Some(UVec2::new(cell_x as u32, cell_y as u32))
}

#[derive(Component)]
struct Highlighted;

#[derive(Component)]
struct HighlightedNeighbor;

fn boid_select_randomly(
    mut commands: Commands,
    boids: Query<(Entity, &Boid), Without<Highlighted>>,
    highlighted: Query<Entity, With<Highlighted>>,
    mouse: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&bevy::window::Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        for entity in highlighted.iter() {
            commands.entity(entity).remove::<Highlighted>();
        }

        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        for entity in highlighted.iter() {
            commands.entity(entity).remove::<Highlighted>();
        }

        let (camera, camera_transform) = camera.single();
        let window = q_windows.single();
        if let Some(mouse) = window.cursor_position() {
            if let Some(mouse_ray) = camera.viewport_to_world(camera_transform, mouse) {
                if let Some((_, entity)) = boids
                    .iter()
                    .map(|(entity, boid)| (boid.position.distance(mouse_ray.origin.xy()), entity))
                    .min_by(|(a, _), (b, _)| a.total_cmp(b))
                {
                    commands.entity(entity).insert(Highlighted);
                }
            }
        }

        return;
    }
}

fn highlight_boid(
    highlighted: Query<(Entity, &Boid), With<Highlighted>>,
    config: Query<&BoidConfiguration>,
    mut gizmos: Gizmos,
) {
    let config = config.single();

    let bounds = Rect::from_corners(config.boid_bounds.min * 12.0, config.boid_bounds.max * 12.0);

    let size = config.spatial_hash_size as f32;
    let half_size = size / 2.0;
    let x_cells = (bounds.width() / size).ceil() as u32;
    let y_cells = (bounds.height() / size).ceil() as u32;
    let cell_size = UVec2::new(x_cells, y_cells);

    let radius = config.protected_range.max(config.visible_range);
    let neighbors = (radius / size).ceil() as u32 + 1;

    for (_, boid) in highlighted.iter() {
        gizmos.circle_2d(boid.position, config.visible_range, Color::LIME_GREEN);

        if let Some(cell) = find_cell_position(boid.position, bounds, size) {
            for y in (cell.y - neighbors).clamp(0, cell_size.y)
                ..(cell.y + neighbors).clamp(0, cell_size.y)
            {
                for x in (cell.x - neighbors).clamp(0, cell_size.x)
                    ..(cell.x + neighbors).clamp(0, cell_size.x)
                {
                    gizmos.rect_2d(
                        bounds.min + half_size + Vec2::new(x as f32, y as f32) * size,
                        0.0,
                        Vec2::splat(size),
                        Color::RED.with_a(0.1),
                    );
                }
            }
        }
    }
}

fn boid_flocking_spatial_hash(
    mut commands: Commands,
    mut boids: Query<(Entity, &mut Boid, Option<&Highlighted>)>,
    old_neighbors: Query<Entity, With<HighlightedNeighbor>>,
    config: Query<&BoidConfiguration>,
) {
    for entity in old_neighbors.iter() {
        commands.entity(entity).remove::<HighlightedNeighbor>();
    }

    let config = config.single();

    let table_size = config.total_boids;

    let size = config.spatial_hash_size as f32;

    let mut spatial_hash: HashMap<u32, Vec<(Entity, Vec2, Vec2)>> =
        HashMap::with_capacity(table_size as usize);

    let bounds = Rect::from_corners(config.boid_bounds.min * 12.0, config.boid_bounds.max * 12.0);

    let x_cells = (bounds.width() / size).round() as u32;
    let y_cells = (bounds.height() / size).round() as u32;
    let cell_size = UVec2::new(x_cells, y_cells);

    for (entity, boid, _) in boids.iter() {
        if let Some(cell) = find_cell_position(boid.position, bounds, size) {
            let key = hash_coords(cell.x, cell.y, cell_size.x * cell_size.y);
            let push_val = (entity, boid.position, boid.velocity);

            if let Some(val) = spatial_hash.get_mut(&key) {
                val.push(push_val);
            } else {
                let val = vec![push_val];
                spatial_hash.insert(key, val);
            }
        }
    }

    // for y in 0..y_cells {
    //     for x in 0..x_cells {
    //         gizmos.rect_2d(
    //             config.boid_bounds.min + half_size + Vec2::new(x as f32, y as f32) * size,
    //             0.0,
    //             Vec2::splat(size),
    //             Color::RED,
    //         );
    //     }
    // }

    let radius = config.protected_range.max(config.visible_range);
    let neighbors = (radius / size).ceil() as u32 + 1;

    for (entity, mut boid, highlighted) in boids.iter_mut() {
        if let Some(cell) = find_cell_position(boid.position, bounds, size) {
            let mut results: Vec<(Entity, Vec2, Vec2)> = vec![];
            for y in (cell.y - neighbors).clamp(0, cell_size.y)
                ..(cell.y + neighbors).clamp(0, cell_size.y)
            {
                for x in (cell.x - neighbors).clamp(0, cell_size.x)
                    ..(cell.x + neighbors).clamp(0, cell_size.x)
                {
                    if let Some(subresults) =
                        spatial_hash.get(&hash_coords(x, y, cell_size.x * cell_size.y))
                    {
                        results.extend(subresults);
                    }
                }
            }

            let mut dclose = Vec2::ZERO;
            let mut velocity_avg = Vec2::ZERO;
            let mut position_avg = Vec2::ZERO;
            let mut boids_in_visible_range = 0;

            for (other_entity, other_position, other_velocity) in results {
                if entity == other_entity {
                    continue;
                }

                let distance = boid.position - other_position;
                if distance.length() <= config.protected_range {
                    dclose += distance;
                }

                if distance.length() <= config.visible_range {
                    boids_in_visible_range += 1;
                    velocity_avg += other_velocity;

                    position_avg += other_position;

                    if highlighted.is_some() {
                        commands.entity(other_entity).insert(HighlightedNeighbor);
                    }
                }
            }

            boid.velocity += dclose * config.avoid_factor;

            if boids_in_visible_range > 0 {
                // alignment
                velocity_avg /= boids_in_visible_range as f32;
                boid.velocity =
                    boid.velocity + (velocity_avg - boid.velocity) * config.matching_factor;

                // cohesion
                position_avg /= boids_in_visible_range as f32;
                boid.velocity =
                    boid.velocity + (position_avg - boid.position) * config.centering_factor
            }
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

        boid.velocity = boid
            .velocity
            .clamp_length(config.min_speed, config.max_speed);
    }
}

fn boid_highlight_neighbors(
    neighbor_boids: Query<&Handle<ColorMaterial>, (With<Boid>, Added<HighlightedNeighbor>)>,
    boids: Query<(&Handle<ColorMaterial>, &Boid), Without<HighlightedNeighbor>>,
    mut removed_neighbors: RemovedComponents<HighlightedNeighbor>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in removed_neighbors.read() {
        if let Ok((material_handle, boid)) = boids.get(entity) {
            if let Some(color) = materials.get_mut(material_handle) {
                color.color = boid.initial_color;
            }
        }
    }

    for material_handle in neighbor_boids.iter() {
        if let Some(color) = materials.get_mut(material_handle) {
            color.color = Color::RED;
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

#[derive(Clone, Debug)]
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

    if !config.bounds_gizmo.enabled {
        return;
    }

    let size = config.boid_bounds.max - config.boid_bounds.min;
    let position = config.boid_bounds.min + size * 0.5;
    gizmos.rect_2d(
        position,
        0.0,
        size,
        Color::rgba_from_array(config.bounds_gizmo.color_rgba),
    );
}
