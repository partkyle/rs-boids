use bevy::{ecs::system::Query, gizmos::gizmos::Gizmos, render::color::Color};

use crate::{config::BoidConfiguration, Boid};

pub fn boid_draw_range_gizmos(
    mut gizmos: Gizmos,
    boids: Query<&Boid>,
    config: Query<&BoidConfiguration>,
) {
    let config = config.single();

    for boid in boids.iter() {
        if config.protected_range_gizmo.enabled {
            gizmos.circle_2d(
                boid.position,
                config.protected_range,
                Color::rgba_from_array(config.protected_range_gizmo.color_rgba),
            );
        }

        if config.visible_range_gizmo.enabled {
            gizmos.circle_2d(
                boid.position,
                config.visible_range,
                Color::rgba_from_array(config.visible_range_gizmo.color_rgba),
            );
        }
    }
}
