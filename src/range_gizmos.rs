use bevy::{ecs::system::Query, gizmos::gizmos::Gizmos, render::color::Color};

use crate::{config::BoidConfiguration, Boid};

pub fn boid_draw_range_gizmos(
    mut gizmos: Gizmos,
    boids: Query<&Boid>,
    config: Query<&BoidConfiguration>,
) {
    let config = config.single();

    for boid in boids.iter() {
        if config.render_protected_range {
            gizmos.circle_2d(
                boid.position,
                config.protected_range,
                Color::RED.with_a(0.1),
            );
        }

        if config.render_visible_range {
            gizmos.circle_2d(
                boid.position,
                config.visible_range,
                Color::YELLOW_GREEN.with_a(0.1),
            );
        }
    }
}
