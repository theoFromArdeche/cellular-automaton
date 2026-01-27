use rand::Rng;


/// Represents a 2D grid of cells (row-major, flat)
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub num_cells: usize,
    pub num_traits: usize,
    /// Single contiguous allocation: [trait0..., trait1..., ..., trait8...]
    pub data: Vec<f32>,
    pub is_empty: Vec<bool>,
}

impl Grid {
    /// Create a new grid with random cells
    pub fn new(width: usize, height: usize, num_traits: usize) -> Self {
        let trait_ranges = vec![(0.0, 1.0); num_traits];
        Self::new_with_density(width, height, 1.0, num_traits, &trait_ranges)
    }

    pub fn new_with_density(width: usize,
                            height: usize,
                            fill_percentage: f32,
                            num_traits: usize,
                            trait_ranges: &Vec<(f32, f32)>,  // (min, max) for each trait
                            ) -> Self {

        let fill_percentage = fill_percentage.clamp(0.0, 1.0);
        let mut rng = rand::thread_rng();
        let num_cells = width * height;
        
        // Single allocation for all traits
        let mut data = vec![0.0; num_cells * num_traits];
        let mut is_empty = vec![true; num_cells]; // Start all empty
        
        for idx in 0..num_cells {
            if rng.gen_range(0.0..=1.0) < fill_percentage {
                is_empty[idx] = false; // Mark as filled
                
                for t in 0..num_traits {
                    let (min, max) = trait_ranges[t];
                    data[t * num_cells + idx] = rng.gen_range(min..=max);
                }
            }
        }
        
        Self {
            width,
            height,
            num_cells,
            num_traits,
            data,
            is_empty,
        }
    }

    #[inline(always)]
    pub fn get_trait_slice(&self, trait_idx: usize) -> &[f32] {
        let start = trait_idx * self.num_cells;
        unsafe { self.data.get_unchecked(start..start + self.num_cells) }
    }

    #[inline(always)]
    pub fn get_trait_slice_mut(&mut self, trait_idx: usize) -> &mut [f32] {
        let start = trait_idx * self.num_cells;
        unsafe { self.data.get_unchecked_mut(start..start + self.num_cells) }
    }


    #[inline(always)]
    fn idx(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    #[inline(always)]
    fn wrap(&self, v: isize, max: usize) -> usize {
        if v < 0 {
            (v + max as isize) as usize
        } else if v >= max as isize {
            (v - max as isize) as usize
        } else {
            v as usize
        }
    }

    #[inline(always)]
    pub fn get_position(&self, row: isize, col: isize) -> (usize, usize) {
        (
            self.wrap(row, self.height),
            self.wrap(col, self.width),
        )
    }

    #[inline(always)]
    pub fn is_cell_empty(&self, row: usize, col: usize) -> bool {
        self.is_empty[self.idx(row, col)]
    }

    #[inline(always)]
    pub fn get_cell_trait(&self, row: usize, col: usize, trait_idx: usize) -> f32 {
        unsafe {
            *self.data.get_unchecked(trait_idx * self.num_cells + self.idx(row, col))
        }
    }

    #[inline(always)]
    pub fn set_cell_trait(&mut self, row: usize, col: usize, trait_idx: usize, value: f32) {
        let pos = self.idx(row, col);
        unsafe {
            *self.data.get_unchecked_mut(trait_idx * self.num_cells + pos) = value;
        }
    }


    /// Fast update that swaps the internal vector (avoids reallocation)
    #[inline(always)]
    pub fn update_grid(&mut self, new_grid: &mut Grid) {
        std::mem::swap(&mut self.data, &mut new_grid.data);
        std::mem::swap(&mut self.is_empty, &mut new_grid.is_empty);
    }

    pub fn count_filled_cells(&self) -> usize {
        self.is_empty.iter().filter(|&&empty| !empty).count()
    }

    pub fn get_fill_percentage(&self) -> f32 {
        let total = self.width * self.height;
        if total == 0 {
            0.0
        } else {
            self.count_filled_cells() as f32 / total as f32
        }
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        
        for r in 0..self.width {
            for c in 0..self.height {
                if !self.is_cell_empty(r, c) {  // false = filled
                    for t in 0..self.num_traits {
                        self.set_cell_trait(r, c, t, rng.gen_range(0.0..=1.0));
                    }
                }
            }
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(5, 5);
        assert_eq!(grid.width, 5);
        assert_eq!(grid.height, 5);
        assert_eq!(grid.data.len(), 5*5*grid.num_traits);
    }

    #[test]
    fn test_wrapping() {
        let grid = Grid::new(5, 5);
        let pos = grid.get_position(-1, -1);
        assert_eq!(pos, (4, 4));
    }

    #[test]
    fn test_grid_with_density() {
        let grid = Grid::new_with_density(1000, 1000, 0.5);
        let fill_percentage = grid.get_fill_percentage();
        assert!(fill_percentage >= 0.45 && fill_percentage <= 0.55);
    }

    #[test]
    fn test_fully_empty_grid() {
        let grid = Grid::new_with_density(5, 5, 0.0);
        assert_eq!(grid.count_filled_cells(), 0);
        assert_eq!(grid.get_fill_percentage(), 0.0);
    }

    #[test]
    fn test_fully_filled_grid() {
        let grid = Grid::new_with_density(5, 5, 1.0);
        assert_eq!(grid.count_filled_cells(), 25);
        assert_eq!(grid.get_fill_percentage(), 1.0);
    }
}