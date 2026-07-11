use crate::generators::random_galaxy::vec2f::Vec2f;

use std::mem;

pub struct QuadTree<const MAX_POINTS: usize, T> {
    items: Vec<QuadTreeEntry<T>>,
    leaf: QuadTreeLeaf<MAX_POINTS>,
    head: Option<usize>,
}

struct QuadTreeLeaf<const MAX_POINTS: usize> {
    bounding_box: BoundingBox,
    points: Vec<(usize, Vec2f)>,
    leaves: Vec<Self>,
}

enum QuadTreeEntry<T> {
    Some(T),
    None { next_free: Option<usize> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    top_left: Vec2f,
    size: Vec2f,
}

impl<const MAX_POINTS: usize, T> QuadTree<MAX_POINTS, T> {
    #[must_use]
    pub const fn new(bounding_box: BoundingBox) -> Self {
        Self {
            items: vec![],
            leaf: QuadTreeLeaf::new(bounding_box),
            head: None,
        }
    }

    #[must_use]
    pub const fn bounding_box(&self) -> BoundingBox {
        self.leaf.bounding_box()
    }

    pub fn remove(&mut self, bounding_box: BoundingBox) -> Vec<(T, Vec2f)> {
        let points_removed = self.leaf.remove(bounding_box);

        let mut items_removed = vec![];

        for (item_index, point) in points_removed {
            if let Some(item) = self.items.get_mut(item_index)
                && let QuadTreeEntry::Some(item) = mem::replace(
                    item,
                    QuadTreeEntry::None {
                        next_free: self.head.take(),
                    },
                )
            {
                self.head = Some(item_index);

                items_removed.push((item, point));
            }
        }

        items_removed
    }

    pub fn insert(&mut self, item: T, point: Vec2f) {
        if self.leaf.insert(self.items.len(), point) {
            if let Some(head) = self.head.take()
                && let Some(QuadTreeEntry::None { next_free }) = self.items.get_mut(head)
            {
                let next_free = next_free.take();

                self.items[head] = QuadTreeEntry::Some(item);

                self.head = next_free;
            } else {
                self.items.push(QuadTreeEntry::Some(item));
            }
        }
    }

    #[must_use]
    pub fn query(&self, bounding_box: BoundingBox) -> Vec<(&T, Vec2f)> {
        self.leaf
            .query(bounding_box)
            .into_iter()
            .filter_map(|(item_index, point)| {
                self.items
                    .get(item_index)
                    .and_then(|maybe_item| match maybe_item {
                        QuadTreeEntry::Some(item) => Some((item, point)),
                        QuadTreeEntry::None { .. } => None,
                    })
            })
            .collect::<Vec<_>>()
    }

    #[must_use]
    pub fn neighbors(&self, point: Vec2f) -> Vec<(&T, Vec2f)> {
        self.leaf
            .neighbors(point)
            .into_iter()
            .filter_map(|(item_index, point)| {
                self.items
                    .get(item_index)
                    .and_then(|maybe_item| match maybe_item {
                        QuadTreeEntry::Some(item) => Some((item, point)),
                        QuadTreeEntry::None { .. } => None,
                    })
            })
            .collect::<Vec<_>>()
    }
}

impl<const MAX_POINTS: usize> QuadTreeLeaf<MAX_POINTS> {
    const fn new(bounding_box: BoundingBox) -> Self {
        Self {
            bounding_box,
            points: vec![],
            leaves: vec![],
        }
    }

    const fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    fn remove(&mut self, bounding_box: BoundingBox) -> Vec<(usize, Vec2f)> {
        let mut found = vec![];

        self.remove_ext(bounding_box, &mut found);

        found
    }

    fn remove_ext(&mut self, bounding_box: BoundingBox, found: &mut Vec<(usize, Vec2f)>) {
        for point_index in (0..(self.points.len())).rev() {
            if let Some((item_index, point)) = self.points.get(point_index)
                && bounding_box.contains(*point)
            {
                found.push((*item_index, *point));

                self.points.swap_remove(point_index);
            }
        }

        for leaf in &mut self.leaves {
            if leaf.bounding_box.intersects(bounding_box) {
                leaf.remove_ext(bounding_box, found);
            }
        }
    }

    fn insert(&mut self, index: usize, point: Vec2f) -> bool {
        if !self.bounding_box.contains(point) {
            return false;
        }

        if self.leaves.is_empty() {
            self.points.push((index, point));

            if self.points.len() >= MAX_POINTS {
                self.subdivide();
            }

            return true;
        }

        for leaf in &mut self.leaves {
            if leaf.insert(index, point) {
                return true;
            }
        }

        false
    }

    fn subdivide(&mut self) {
        self.leaves.push(Self::new(BoundingBox::new(
            self.bounding_box.top_left(),
            self.bounding_box.size() / 2.0,
        )));

        self.leaves.push(Self::new(BoundingBox::new(
            Vec2f::new(
                self.bounding_box.top_left().x() + self.bounding_box.size().x() / 2.0,
                *self.bounding_box.top_left().y(),
            ),
            self.bounding_box.size() / 2.0,
        )));

        self.leaves.push(Self::new(BoundingBox::new(
            Vec2f::new(
                *self.bounding_box.top_left().x(),
                self.bounding_box.top_left().y() + self.bounding_box.size().y() / 2.0,
            ),
            self.bounding_box.size() / 2.0,
        )));

        self.leaves.push(Self::new(BoundingBox::new(
            self.bounding_box.top_left() + self.bounding_box.size() / 2.0,
            self.bounding_box.size() / 2.0,
        )));

        while let Some((index, point)) = self.points.pop() {
            for leaf in &mut self.leaves {
                if leaf.insert(index, point) {
                    break;
                }
            }
        }
    }

    fn query(&self, bounding_box: BoundingBox) -> Vec<(usize, Vec2f)> {
        let mut found = vec![];

        self.query_ext(bounding_box, &mut found);

        found
    }

    fn query_ext(&self, bounding_box: BoundingBox, found: &mut Vec<(usize, Vec2f)>) {
        for (index, point) in &self.points {
            if bounding_box.contains(*point) {
                found.push((*index, *point));
            }
        }

        for leaf in &self.leaves {
            if leaf.bounding_box.intersects(bounding_box) {
                leaf.query_ext(bounding_box, found);
            }
        }
    }

    fn neighbors(&self, point: Vec2f) -> Vec<(usize, Vec2f)> {
        let mut found = vec![];
        let mut within = None;

        self.neighbors_ext(point, &mut within);

        if let Some(within) = within {
            self.nearest_neighbors(&mut found, within);
        }

        found
    }

    fn neighbors_ext<'a>(&'a self, point: Vec2f, within: &mut Option<&'a Self>) {
        if self.bounding_box.contains(point) {
            if self.leaves.is_empty() && within.is_none() {
                *within = Some(self);
            } else {
                for leaf in &self.leaves {
                    leaf.neighbors_ext(point, within);
                }
            }
        }
    }

    fn nearest_neighbors<'a>(&'a self, found: &mut Vec<(usize, Vec2f)>, within: &'a Self) {
        if self.leaves.is_empty() {
            for (index, point) in &self.points {
                found.push((*index, *point));
            }
        }

        let mut closest_tree = None;

        for leaf in &self.leaves {
            if !leaf.points.is_empty() || !leaf.leaves.is_empty() {
                let leaf_distance = (within.bounding_box.top_left + within.bounding_box.size / 2.0)
                    .distance(leaf.bounding_box.top_left + leaf.bounding_box.size / 2.0);

                if closest_tree.is_none_or(|(distance, _)| distance > leaf_distance) {
                    closest_tree = Some((leaf_distance, leaf));
                }
            }
        }

        if let Some((_, leaf)) = closest_tree {
            leaf.nearest_neighbors(found, within);
        }
    }
}

impl BoundingBox {
    #[must_use]
    pub const fn new(top_left: Vec2f, size: Vec2f) -> Self {
        Self { top_left, size }
    }

    #[must_use]
    pub const fn top_left(&self) -> Vec2f {
        self.top_left
    }

    #[must_use]
    pub const fn size(&self) -> Vec2f {
        self.size
    }

    #[must_use]
    pub const fn contains(&self, point: Vec2f) -> bool {
        point.x >= *self.top_left().x()
            && point.y >= *self.top_left().y()
            && point.x < *self.top_left().x() + *self.size().x()
            && point.y < *self.top_left().y() + *self.size().y()
    }

    #[must_use]
    pub const fn intersects(&self, other: Self) -> bool {
        *other.top_left().x() <= *self.top_left().x() + *self.size().x()
            && *other.top_left().y() <= *self.top_left().y() + *self.size().y()
            && *other.top_left().x() + *other.size().x() >= *self.top_left().x()
            && *other.top_left().y() + *other.size().y() >= *self.top_left().y()
    }
}
