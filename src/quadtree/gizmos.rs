use bevy::{
    color::Color,
    ecs::system::{Query, Res},
    gizmos::gizmos::Gizmos,
    math::Isometry2d,
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

        let iso = Isometry2d::from_translation(origin);

        let color = Color::srgba(
            config.quadtree_gizmo.color_rgba[0],
            config.quadtree_gizmo.color_rgba[1],
            config.quadtree_gizmo.color_rgba[2],
            config.quadtree_gizmo.color_rgba[3],
        );

        gizmos.rect_2d(iso, size, color);
    }
}
