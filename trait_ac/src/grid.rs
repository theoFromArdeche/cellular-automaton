use rand::Rng;



/// Represents a 2D grid of cells (row-major, flat)
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub traits: [Vec<f32>; 9],
    pub is_empty: Vec<u8>,
}

impl Grid {
    /// Create a new grid with random cells
    pub fn new(width: usize, height: usize) -> Self {
        Self::new_with_density(width, height, 1.0)
    }

    /// Create a new grid with a specified density of filled cells
    /// 
    /// # Arguments
    /// * `width` - Width of the grid
    /// * `height` - Height of the grid
    /// * `fill_percentage` - Percentage of cells to fill (0.0 to 1.0)
    ///   - 1.0 = 100% filled (all cells have random values)
    ///   - 0.5 = 50% filled, 50% empty
    ///   - 0.0 = 0% filled (all cells are empty)
    pub fn new_with_density(width: usize, height: usize, fill_percentage: f32) -> Self {
        let fill_percentage = fill_percentage.clamp(0.0, 1.0);
        let mut rng = rand::thread_rng();
        let len = width * height;

        let mut traits: [Vec<f32>; 9] = std::array::from_fn(|_| vec![0.0; len]);
        let mut is_empty = vec![1u8; len];

        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if rng.gen_range(0.0..=1.0) < fill_percentage {
                    is_empty[idx] = 0;
                    for t in 0..9 {
                        traits[t][idx] = rng.gen_range(0.0..=1.0);
                    }
                }
            }
        }

        Self {
            width,
            height,
            traits,
            is_empty,
        }
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
    pub fn is_cell_empty(&self, row: usize, col: usize) -> u8 {
        unsafe {
            *self.is_empty.get_unchecked(self.idx(row, col))
        }
    }

    #[inline(always)]
    pub fn get_cell_trait(&self, row: usize, col: usize, trait_idx: usize) -> f32 {
        unsafe {
            *self.traits[trait_idx].get_unchecked(self.idx(row, col))
        }
    }

    #[inline(always)]
    pub fn get_wrapped_cell_trait( &self, row: isize, col: isize, trait_idx: usize) -> f32 {
        let (r, c) = self.get_position(row, col);
        self.get_cell_trait(r, c, trait_idx)
    }

    #[inline(always)]
    pub fn is_wrapped_empty(&self, row: isize, col: isize) -> u8 {
        let (r, c) = self.get_position(row, col);
        self.is_cell_empty(r, c)
    }


    #[inline(always)]
    pub fn set_cell_trait(&mut self, row: usize, col: usize, trait_idx: usize, value: f32) {
        let pos = self.idx(row, col);
        unsafe {
            *self.traits[trait_idx].get_unchecked_mut(pos) = value;
        }
    }

    #[inline(always)]
    pub fn set_empty(&mut self, row: usize, col: usize, empty: bool) {
        let pos = self.idx(row, col);
        unsafe {
            *self.is_empty.get_unchecked_mut(pos) = empty as u8;
        }
    }


    /// Fast update that swaps the internal vector (avoids reallocation)
    #[inline(always)]
    pub fn update_grid(&mut self, new_grid: &mut Grid) {
        std::mem::swap(&mut self.traits, &mut new_grid.traits);
        std::mem::swap(&mut self.is_empty, &mut new_grid.is_empty);
    }

    /// Get trait values for all cells in row-major order
    #[inline]
    pub fn get_cell_trait_array(&self, trait_index: usize) -> Vec<f32> {
        self.traits[trait_index].clone()
    }


    pub fn count_filled_cells(&self) -> usize {
        self.is_empty.iter().filter(|&&e| e == 0).count()
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
        for i in 0..self.traits[0].len() {
            if self.is_empty[i] == 0 {
                for t in 0..9 {
                    self.traits[t][i] = rng.gen_range(0.0..=1.0);
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
        assert_eq!(grid.traits.len(), 5*5);
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