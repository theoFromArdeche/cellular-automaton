use crate::cell::Cell;

/// Represents a 2D grid of cells
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
}

impl Grid {
    /// Create a new grid with random cells
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(height);
        for row in 0..height {
            let mut row_cells = Vec::with_capacity(width);
            for col in 0..width {
                row_cells.push(Cell::random((row, col)));
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
    pub fn get_position(&self, row: isize, col: isize) -> (usize, usize) {
        let wrapped_row = ((row % self.height as isize + self.height as isize) % self.height as isize) as usize;
        let wrapped_col = ((col % self.width as isize + self.width as isize) % self.width as isize) as usize;
        (wrapped_row, wrapped_col)
    }


    /// Get a cell at position (row, col), wrapping around (toroidal grid)
    pub fn get_cell(&self, row: isize, col: isize) -> &Cell {
        let (wrapped_row, wrapped_col) = self.get_position(row, col);
        &self.cells[wrapped_row][wrapped_col]
    }

    /// Get all cells in the grid as a flat vector
    pub fn get_all_cells(&self) -> Vec<&Cell> {
        self.cells.iter().flat_map(|row| row.iter()).collect()
    }

    /// Update the grid with new cell states
    pub fn update_cells(&mut self, new_cells: Vec<Vec<Cell>>) {
        self.cells = new_cells;
    }

    /// Get trait values for all cells in row-major order
    pub fn get_trait_array(&self, trait_index: usize) -> Vec<f32> {
        self.cells
            .iter()
            .flat_map(|row| row.iter().map(|cell| cell.get_trait(trait_index)))
            .collect()
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
}
