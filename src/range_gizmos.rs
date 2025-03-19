use bevy::{color::Color, ecs::system::Query, gizmos::gizmos::Gizmos};

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
                Color::linear_rgba(
                    config.protected_range_gizmo.color_rgba[0],
                    config.protected_range_gizmo.color_rgba[1],
                    config.protected_range_gizmo.color_rgba[2],
                    config.protected_range_gizmo.color_rgba[3],
                ),
            );
        }

        if config.visible_range_gizmo.enabled {
            gizmos.circle_2d(
                boid.position,
                config.visible_range,
                Color::linear_rgba(
                    config.protected_range_gizmo.color_rgba[0],
                    config.protected_range_gizmo.color_rgba[1],
                    config.protected_range_gizmo.color_rgba[2],
                    config.protected_range_gizmo.color_rgba[3],
                ),
            );
        }
    }
}
