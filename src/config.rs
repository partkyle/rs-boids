use bevy::{
    ecs::component::Component,
    math::{Rect, Vec2},
};

#[derive(Component, Debug)]
pub struct BoidConfiguration {
    pub total_boids: u32,
    pub spawn_count: u32,
    pub spawn_range: Rect,
    pub turn_factor: f32,
    pub boid_bounds: Rect,
    pub visible_range: f32,
    pub protected_range: f32,
    pub avoid_factor: f32,
    pub centering_factor: f32,
    pub matching_factor: f32,
    pub max_speed: f32,
    pub min_speed: f32,

    pub spatial_hash_size: u32,

    pub bounds_gizmo: BoidGizmoConfig,
    pub quadtree_gizmo: BoidGizmoConfig,
    pub protected_range_gizmo: BoidGizmoConfig,
    pub visible_range_gizmo: BoidGizmoConfig,

    pub update_color_sample_rate: f32,
    pub update_color_type: ColorType,
}

impl Default for BoidConfiguration {
    fn default() -> Self {
        BoidConfiguration {
            total_boids: 0,
            spawn_count: 100,
            spawn_range: Rect {
                min: Vec2::new(-200.0, -200.0),
                max: Vec2::new(200.0, 200.0),
            },

            boid_bounds: Rect {
                min: Vec2::new(-200.0, -200.0),
                max: Vec2::new(200.0, 200.0),
            },

            turn_factor: 1.2,

            visible_range: 100.0,
            protected_range: 40.0,

            centering_factor: 0.0005,
            avoid_factor: 0.05,
            matching_factor: 0.05,

            max_speed: 100.0,
            min_speed: 2.0,

            spatial_hash_size: 100,

            bounds_gizmo: BoidGizmoConfig::new(false, [0.8, 0.6, 0.8, 1.0]),
            quadtree_gizmo: BoidGizmoConfig::new(false, [0.0, 1.0, 0.0, 0.1]),
            protected_range_gizmo: BoidGizmoConfig::new(false, [1.0, 0.0, 0.0, 0.1]),
            visible_range_gizmo: BoidGizmoConfig::new(false, [0.6, 1.0, 0.0, 0.1]),

            update_color_sample_rate: 0.0,
            update_color_type: ColorType::Synthwave,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ColorType {
    Initial,
    Synthwave,
    Pastel,
    PrimaryRGB,
}

#[derive(Default, Debug)]
pub struct BoidGizmoConfig {
    pub enabled: bool,
    pub color_rgba: [f32; 4],
}

impl BoidGizmoConfig {
    pub fn new(enabled: bool, color_rgba: [f32; 4]) -> Self {
        Self {
            enabled,
            color_rgba,
        }
    }
}
