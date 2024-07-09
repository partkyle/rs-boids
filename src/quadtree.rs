use bevy::math::{Rect, Vec2};

#[derive(Debug)]
pub struct Quadtree<T: Clone + std::fmt::Debug> {
    boundary: Rect,
    capacity: usize,
    points: Vec<(Vec2, T)>,
    quadrants: Option<Quadrants<Box<Quadtree<T>>>>,
    count: usize,
}

#[derive(Debug)]
struct Quadrants<T>(T, T, T, T);

impl<T> Quadrants<T> {
    fn nw(&self) -> &T {
        &self.0
    }
    fn nw_mut(&mut self) -> &mut T {
        &mut self.0
    }

    fn ne(&self) -> &T {
        &self.1
    }
    fn ne_mut(&mut self) -> &mut T {
        &mut self.1
    }

    fn sw(&self) -> &T {
        &self.2
    }
    fn sw_mut(&mut self) -> &mut T {
        &mut self.2
    }

    fn se(&self) -> &T {
        &self.3
    }
    fn se_mut(&mut self) -> &mut T {
        &mut self.3
    }
}

impl<T: Clone + std::fmt::Debug> Quadtree<T> {
    pub fn new(boundary: Rect, capacity: usize) -> Self {
        Quadtree {
            boundary,
            capacity,
            points: Vec::new(),
            quadrants: None,
            count: 0,
        }
    }

    fn subdivide(&mut self) {
        let min = self.boundary.min;
        let max = self.boundary.max;
        let mid = self.boundary.min + (max - min) / 2.0;

        let nw_boundary = Rect {
            min: Vec2::new(min.x, mid.y),
            max: Vec2::new(mid.x, max.y),
        };
        let ne_boundary = Rect {
            min: Vec2::new(mid.x, mid.y),
            max: Vec2::new(max.x, max.y),
        };
        let sw_boundary = Rect {
            min: Vec2::new(min.x, min.y),
            max: Vec2::new(mid.x, mid.y),
        };
        let se_boundary = Rect {
            min: Vec2::new(mid.x, min.y),
            max: Vec2::new(max.x, mid.y),
        };

        self.quadrants = Some(Quadrants(
            Box::new(Quadtree::new(nw_boundary, self.capacity)),
            Box::new(Quadtree::new(ne_boundary, self.capacity)),
            Box::new(Quadtree::new(sw_boundary, self.capacity)),
            Box::new(Quadtree::new(se_boundary, self.capacity)),
        ));
    }

    fn contains(&self, point: Vec2) -> bool {
        self.boundary.min.x <= point.x
            && point.x < self.boundary.max.x
            && self.boundary.min.y <= point.y
            && point.y < self.boundary.max.y
    }

    pub fn insert(&mut self, point: Vec2, value: T) {
        if !self.contains(point) {
            return;
        }

        if self.points.len() < self.capacity {
            self.count += 1;
            self.points.push((point, value.clone()));
        } else {
            if self.quadrants.is_none() {
                self.subdivide();
            }

            if let Some(quadrant) = &mut self.quadrants {
                quadrant.nw_mut().insert(point, value.clone());
                quadrant.ne_mut().insert(point, value.clone());
                quadrant.sw_mut().insert(point, value.clone());
                quadrant.se_mut().insert(point, value.clone());
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_count(&self) -> usize {
        self.count
            + match &self.quadrants {
                None => 0,
                Some(quadrants) => {
                    quadrants.0.get_count()
                        + quadrants.1.get_count()
                        + quadrants.2.get_count()
                        + quadrants.3.get_count()
                }
            }
    }

    pub fn query(&self, range: Rect) -> Vec<(Vec2, T)> {
        let mut result = vec![];
        self.query_internal(range, &mut result);
        result
    }

    // Function to query points within a range
    fn query_internal(&self, range: Rect, found_points: &mut Vec<(Vec2, T)>) {
        if self.boundary.intersect(range).is_empty() {
            return;
        }

        if self.points.len() > 0 {
            for (point, data) in self.points.iter().cloned() {
                if range.contains(point) {
                    found_points.push((point, data));
                }
            }
        }

        if let Some(quadrants) = &self.quadrants {
            quadrants.nw().query_internal(range, found_points);
            quadrants.ne().query_internal(range, found_points);
            quadrants.sw().query_internal(range, found_points);
            quadrants.se().query_internal(range, found_points);
        }
    }

    pub fn get_all_bounds(&self) -> Vec<Rect> {
        match &self.quadrants {
            None => vec![self.boundary],
            Some(quadrants) => {
                let mut bounds = vec![];
                bounds.extend(quadrants.nw().get_all_bounds());
                bounds.extend(quadrants.ne().get_all_bounds());
                bounds.extend(quadrants.sw().get_all_bounds());
                bounds.extend(quadrants.se().get_all_bounds());
                bounds
            }
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.quadrants = None;
    }
}

#[cfg(test)]
mod test {
    use crate::quadtree::Quadtree;
    use bevy::math::{Rect, Vec2};

    #[test]
    fn test_quadtree() {
        let mut q = Quadtree::new(
            Rect {
                min: Vec2 {
                    x: -100.0,
                    y: -100.0,
                },
                max: Vec2 { x: 100.0, y: 100.0 },
            },
            1,
        );

        q.insert(Vec2 { x: 0.0, y: 0.0 }, 1);
        q.insert(Vec2 { x: 0.0, y: 1.0 }, 2);

        assert_eq!(q.get_count(), 2, "should have count of 2");

        let results = q.query(Rect {
            min: Vec2 {
                x: -100.0,
                y: -100.0,
            },
            max: Vec2 { x: 100.0, y: 100.0 },
        });

        assert_eq!(results.len(), 2, "should query results of 2");
    }

    #[test]
    fn larger_test() {
        let mut q = Quadtree::new(
            Rect {
                min: Vec2 {
                    x: -100.0,
                    y: -100.0,
                },
                max: Vec2 { x: 100.0, y: 100.0 },
            },
            1,
        );

        for i in 0..50 {
            q.insert(Vec2::splat(-50.0), i);
        }

        let results = q.query(Rect::new(-51.0, -51.0, -49.0, -49.0));

        assert_eq!(q.get_count(), 50, "count");
        assert_eq!(results.len(), 50, "results");
    }
}
