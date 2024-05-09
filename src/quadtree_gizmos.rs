use bevy::{
    ecs::system::{Query, Res},
    gizmos::gizmos::Gizmos,
    render::color::Color,
};

use crate::{config::BoidConfiguration, QuadtreeJail};

pub fn render_quadtree(
    config: Query<&BoidConfiguration>,
    qt: Res<QuadtreeJail>,
    mut gizmos: Gizmos,
) {
    let config = config.single();

    if !config.quadtree_gizmo.enabled {
        return;
    }

    for b in qt.get_all_bounds() {
        let size = b.max - b.min;
        let origin = b.min + size * 0.5;

        gizmos.rect_2d(
            origin,
            0.0,
            size,
            Color::rgba_from_array(config.quadtree_gizmo.color_rgba),
        );
    }
}
