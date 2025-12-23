use crate::cell::Cell;
use rand::Rng;



/// Represents a 2D grid of cells
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
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
        let mut cells = Vec::with_capacity(height);

        for row in 0..height {
            let mut row_cells = Vec::with_capacity(width);
            for col in 0..width {
                // Randomly decide if this cell should be filled based on percentage
                if rng.gen_range(0.0..=1.0) < fill_percentage {
                    row_cells.push(Cell::random((row, col)));
                } else {
                    row_cells.push(Cell::empty_at((row, col)));
                }
            }
            cells.push(row_cells);
        }

        Self {
            width,
            height,
            cells,
        }
    }

    /// Get a position (row, col), wrapping around (toroidal grid)
    #[inline(always)]
    pub fn get_position(&self, row: isize, col: isize) -> (usize, usize) {
        let wrapped_row = row.rem_euclid(self.height as isize) as usize;
        let wrapped_col = col.rem_euclid(self.width as isize) as usize;
        (wrapped_row, wrapped_col)
    }

    /// Get a cell at position (row, col), wrapping around (toroidal grid)
    #[inline(always)]
    pub fn get_cell(&self, row: isize, col: isize) -> &Cell {
        let (wrapped_row, wrapped_col) = self.get_position(row, col);
        unsafe {
            self.cells.get_unchecked(wrapped_row).get_unchecked(wrapped_col)
        }
    }

    /// Get all cells in the grid as a flat vector
    pub fn get_all_cells(&self) -> Vec<&Cell> {
        self.cells.iter().flat_map(|row| row.iter()).collect()
    }

    /// Update the grid with new cell states
    #[inline(always)]
    pub fn update_cells(&mut self, new_cells: Vec<Vec<Cell>>) {
        self.cells = new_cells;
    }

    /// Fast update that swaps the internal vector (avoids reallocation)
    #[inline(always)]
    pub fn update_cells_fast(&mut self, new_cells: &mut Vec<Vec<Cell>>) {
        std::mem::swap(&mut self.cells, new_cells);
    }

    /// Get trait values for all cells in row-major order
    pub fn get_trait_array(&self, trait_index: usize) -> Vec<f32> {
        self.cells
            .iter()
            .flat_map(|row| row.iter().map(|cell| cell.get_trait(trait_index)))
            .collect()
    }

    /// Randomize grid
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for row in &mut self.cells {
            for cell in row {
                for i in 0..cell.fingerprint.len() {
                    cell.set_trait(i, rng.gen_range(0.0..=1.0));
                }
            }
        }
    }

    /// Count the number of filled (non-empty) cells
    pub fn count_filled_cells(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| !cell.is_empty())
            .count()
    }

    /// Get the actual fill percentage of the grid
    pub fn get_fill_percentage(&self) -> f32 {
        let total_cells = self.width * self.height;
        if total_cells == 0 {
            return 0.0;
        }
        self.count_filled_cells() as f32 / total_cells as f32
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
        assert_eq!(grid.cells.len(), 5);
        assert_eq!(grid.cells[0].len(), 5);
    }

    #[test]
    fn test_wrapping() {
        let grid = Grid::new(5, 5);
        let cell = grid.get_cell(-1, -1);
        assert_eq!(cell.position, (4, 4));
    }

    #[test]
    fn test_grid_with_density() {
        let grid = Grid::new_with_density(100, 100, 0.5);
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