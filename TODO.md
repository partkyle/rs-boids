


https://github.com/bevyengine/bevy/discussions/6907

this might be the right way to do the query for the quadtree

that would be awesome if we learned that.

```
fn change_material(
    enemies: Query<(&Children, &Enemy)>,
    mut standard_materials: Query<&mut Handle<StandardMaterial>> ,
) { 
    for (children, enemy) in &enemies {
        for child in children {
            if let Ok(handle) = standard_materials.get_mut(*child) {
                // etc
            }
        }
    }

    ```