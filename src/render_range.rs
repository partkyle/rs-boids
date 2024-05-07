use bevy::{
    ecs::{component::Component, system::Query},
    render::view::Visibility,
};
use bevy_prototype_lyon::{entity::Path, geometry::GeometryBuilder, shapes};

use crate::BoidConfiguration;

pub trait RangeVisibility {
    fn is_visible(&self, config: &BoidConfiguration) -> bool;
}

pub trait GetRangeRadius {
    fn radius(&self, config: &BoidConfiguration) -> f32;
}

#[derive(Component, Default)]
pub struct BoidProtectedRange;

impl RangeVisibility for BoidProtectedRange {
    fn is_visible(&self, config: &BoidConfiguration) -> bool {
        config.render_protected_range
    }
}

impl GetRangeRadius for BoidProtectedRange {
    fn radius(&self, config: &BoidConfiguration) -> f32 {
        config.protected_range
    }
}

#[derive(Component, Default)]
pub struct BoidVisibleRange;

impl RangeVisibility for BoidVisibleRange {
    fn is_visible(&self, config: &BoidConfiguration) -> bool {
        config.render_visible_range
    }
}

impl GetRangeRadius for BoidVisibleRange {
    fn radius(&self, config: &BoidConfiguration) -> f32 {
        config.visible_range
    }
}

pub fn boid_update_range_visibility<T: Component + RangeVisibility>(
    config: Query<&BoidConfiguration>,
    mut ranges: Query<(&mut Visibility, &T)>,
) {
    let config = config.single();
    for (mut visibility, should_render) in ranges.iter_mut() {
        if should_render.is_visible(&config) {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        };
    }
}

pub fn boid_update_range_path<T: Component + RangeVisibility + GetRangeRadius>(
    config: Query<&BoidConfiguration>,
    mut ranges: Query<(&mut Path, &T)>,
) {
    let config = config.single();
    for (mut path, range_type) in ranges.iter_mut() {
        // early exit, but it's still done in the loop
        // this can still save processing time, but perhaps there's a better way
        if !range_type.is_visible(&config) {
            return;
        }

        *path = GeometryBuilder::build_as(&shapes::Circle {
            radius: range_type.radius(&config),
            ..Default::default()
        });
    }
}
