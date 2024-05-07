use bevy::{
    ecs::component::Component,
    math::{Rect, Vec2},
};

#[derive(Component, Debug)]
pub struct BoidConfiguration {
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

    pub render_quadtree: bool,
    pub render_protected_range: bool,
    pub render_visible_range: bool,

    pub update_color_sample_rate: f32,
    pub update_color_type: ColorType,
}

impl Default for BoidConfiguration {
    fn default() -> Self {
        BoidConfiguration {
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

            render_quadtree: false,
            render_protected_range: false,
            render_visible_range: false,

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
