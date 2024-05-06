use bevy::math::{Rect, Vec2};

#[derive(Debug)]
pub struct Quadtree<T: Clone> {
    boundary: Rect,
    capacity: usize,
    points: Vec<(Vec2, T)>,
    divided: bool,
    northeast: Option<Box<Quadtree<T>>>,
    northwest: Option<Box<Quadtree<T>>>,
    southeast: Option<Box<Quadtree<T>>>,
    southwest: Option<Box<Quadtree<T>>>,
}

impl<T: Clone> Quadtree<T> {
    pub fn new(boundary: Rect, capacity: usize) -> Self {
        Quadtree {
            boundary,
            capacity,
            points: Vec::new(),
            divided: false,
            northeast: None,
            northwest: None,
            southeast: None,
            southwest: None,
        }
    }

    fn subdivide(&mut self) {
        let min = self.boundary.min;
        let max = self.boundary.max;
        let mid = Vec2::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0);

        let ne_boundary = Rect { min: mid, max: max };
        let nw_boundary = Rect { min: Vec2::new(min.x, mid.y), max: Vec2::new(mid.x, max.y) };
        let se_boundary = Rect { min: Vec2::new(mid.x, min.y), max: Vec2::new(max.x, mid.y) };
        let sw_boundary = Rect { min: min, max: mid };

        self.northeast = Some(Box::new(Quadtree::new(ne_boundary, self.capacity)));
        self.northwest = Some(Box::new(Quadtree::new(nw_boundary, self.capacity)));
        self.southeast = Some(Box::new(Quadtree::new(se_boundary, self.capacity)));
        self.southwest = Some(Box::new(Quadtree::new(sw_boundary, self.capacity)));

        self.divided = true;
    }

    pub fn insert(&mut self, point: Vec2, value: T) {
        if !self.boundary.contains(point) {
            return;
        }

        if self.points.len() < self.capacity {
            self.points.push((point, value.clone()));
        } else {
            if !self.divided {
                self.subdivide();
            }

            if let Some(ref mut northeast) = self.northeast {
                northeast.insert(point, value.clone());
            }
            if let Some(ref mut northwest) = self.northwest {
                northwest.insert(point, value.clone());
            }
            if let Some(ref mut southeast) = self.southeast {
                southeast.insert(point, value.clone());
            }
            if let Some(ref mut southwest) = self.southwest {
                southwest.insert(point, value.clone());
            }
        }
    }

    // Function to query points within a range
    pub fn query(&self, range: &Rect, found_points: &mut Vec<(Vec2, T)>) {
        if !self.boundary.contains(range.min) && !self.boundary.contains(range.max) {
            return;
        }

        if self.points.len() > 0 {
            for (point, data) in self.points.iter().cloned() {
                if range.contains(point) {
                    found_points.push((point, data));
                }
            }
        }

        if self.divided {
            self.northwest.as_ref().unwrap().query(range, found_points);
            self.northeast.as_ref().unwrap().query(range, found_points);
            self.southwest.as_ref().unwrap().query(range, found_points);
            self.southeast.as_ref().unwrap().query(range, found_points);
        }
    }

    pub fn get_all_bounds(&self) -> Vec<Rect> {
        if self.northeast.is_none() {
            return vec![self.boundary];
        }

        let mut bounds = vec![];
        if let Some(ref northeast) = self.northeast {
            bounds.extend(northeast.get_all_bounds());
        }
        if let Some(ref northwest) = self.northwest {
            bounds.extend(northwest.get_all_bounds());
        }
        if let Some(ref southeast) = self.southeast {
            bounds.extend(southeast.get_all_bounds());
        }
        if let Some(ref southwest) = self.southwest {
            bounds.extend(southwest.get_all_bounds());
        }

        bounds
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.divided = false;
        self.northeast = None;
        self.northwest = None;
        self.southeast = None;
        self.southwest = None;
    }
}


#[cfg(test)]
mod test {
    use bevy::math::{Rect, Vec2};
    use crate::quadtree::Quadtree;

    #[test]
    fn test_quadtree() {
        let mut q = Quadtree::new(Rect { min: Vec2 { x: -100.0, y: -100.0 }, max: Vec2 { x: 100.0, y: 100.0 } }, 1);

        q.insert(Vec2 { x: 0.0, y: 0.0 }, 1);
        q.insert(Vec2 { x: 0.0, y: 1.0 }, 2);

        let mut results = vec![];
        q.query(&Rect::new(-2.0, -2.0, 2.0, 2.0), &mut results);

        assert_eq!(results.len(), 2);
    }
}