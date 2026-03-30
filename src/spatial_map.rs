use glam::Vec3;

pub struct SpatialMap {
    pub spacial_lookup: Vec<(usize, usize)>,
    pub start_indices: Vec<usize>,
    pub cell_size: f32,
    pub count: usize,
}

impl SpatialMap {
    const P1: isize = 15823;
    const P2: isize = 9739333;
    const P3: isize = 786433;
    pub fn new(cell_size: f32, count: usize) -> Self {
        SpatialMap {
            spacial_lookup: vec![(0, 0); count],
            start_indices: vec![usize::MAX; count],
            cell_size: cell_size,
            count: count,
        }
    }
    pub fn update_params(&mut self, cell_size: f32) {
        self.cell_size = cell_size;
    }
    pub fn coords_to_key(&self, coords: (isize, isize, isize)) -> usize {
        let hash = Self::P1 * coords.0 + Self::P2 * coords.1 + Self::P3 * coords.2;
        return hash as usize % self.count;
    }
    pub fn pos_to_coords(&self, pos: Vec3) -> (isize, isize, isize) {
        let shifted = pos;
        let c_x = (shifted.x / self.cell_size) as isize;
        let c_y = (shifted.y / self.cell_size) as isize;
        let c_z = (shifted.z / self.cell_size) as isize;
        return (c_x, c_y, c_z);
    }
    pub fn pos_to_key(&self, pos: Vec3) -> usize {
        let coords = self.pos_to_coords(pos);
        let key = self.coords_to_key(coords);
        // println!("Pos: {}, coord: {:?}, key: {}", pos, coords, key);

        return key;
    }

    pub fn insert(&mut self, index: usize, pos: Vec3) {
        let cell_key = self.pos_to_key(pos);

        self.spacial_lookup[index] = (cell_key, index);
    }
    pub fn get(&self, pos: Vec3) -> Vec<usize> {
        return self.get_cords(self.pos_to_coords(pos));
    }

    fn get_cords(&self, coords: (isize, isize, isize)) -> Vec<usize> {
        let mut indexes = Vec::new();

        let key = self.coords_to_key(coords);
        let start_index = self.start_indices[key];
        if start_index == usize::MAX {
            return Vec::new();
        }

        let mut i = start_index;
        let mut prev = self.spacial_lookup[i].0;
        while i < self.count && prev == self.spacial_lookup[i].0 {
            indexes.push(self.spacial_lookup[i].1);
            prev = self.spacial_lookup[i].0;

            i += 1;
        }
        return indexes;
    }
    pub fn get_around(&self, pos: Vec3) -> Vec<usize> {
        let offsets: [(isize, isize, isize); 27] = [
            (-1, -1, -1),
            (0, -1, -1),
            (1, -1, -1),
            (-1, 0, -1),
            (0, 0, -1),
            (1, 0, -1),
            (-1, 1, -1),
            (0, 1, -1),
            (1, 1, -1),
            (-1, -1, 0),
            (0, -1, 0),
            (1, -1, 0),
            (-1, 0, 0),
            (0, 0, 0),
            (1, 0, 0),
            (-1, 1, 0),
            (0, 1, 0),
            (1, 1, 0),
            (-1, -1, 1),
            (0, -1, 1),
            (1, -1, 1),
            (-1, 0, 1),
            (0, 0, 1),
            (1, 0, 1),
            (-1, 1, 1),
            (0, 1, 1),
            (1, 1, 1),
        ];

        let coords = self.pos_to_coords(pos);
        let mut master = Vec::new();
        for of in offsets {
            let x: isize = coords.0 as isize + of.0;
            let y: isize = coords.1 as isize + of.1;
            let z: isize = coords.2 as isize + of.2;

            let new_cords: (isize, isize, isize) = (x, y, z);
            master.append(&mut self.get_cords(new_cords));
        }
        return master;
    }

    pub fn finalize(&mut self) {
        self.spacial_lookup.sort_by_key(|p| {
            return p.0;
        });

        let mut i = 0;
        let mut prev = usize::MAX;
        while i < self.count {
            if self.spacial_lookup[i].0 != prev {
                self.start_indices[self.spacial_lookup[i].0] = i;
            }
            prev = self.spacial_lookup[i].0;
            i += 1;
        }
    }
}
