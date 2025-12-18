use crate::cell::Cell;

/// Represents a 2D grid of cells with configurable neighborhood
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
    pub neighborhood_mask: Vec<Vec<bool>>, // 3x3 mask for neighbor selection
}

impl Grid {
    /// Create a new grid with random cells
    pub fn new(width: usize, height: usize, neighborhood_mask: Vec<Vec<bool>>) -> Self {
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
            neighborhood_mask,
        }
    }

    /// Get a cell at position (row, col), wrapping around (toroidal grid)
    pub fn get_cell(&self, row: isize, col: isize) -> &Cell {
        let wrapped_row = ((row % self.height as isize + self.height as isize) % self.height as isize) as usize;
        let wrapped_col = ((col % self.width as isize + self.width as isize) % self.width as isize) as usize;
        &self.cells[wrapped_row][wrapped_col]
    }

    /// Get neighbors of a cell based on the neighborhood mask
    pub fn get_neighbors(&self, row: usize, col: usize) -> Vec<Cell> {
        let mut neighbors = Vec::new();
        
        for mask_row in 0..3 {
            for mask_col in 0..3 {
                if self.neighborhood_mask[mask_row][mask_col] {
                    // Skip center cell
                    if mask_row == 1 && mask_col == 1 {
                        continue;
                    }
                    
                    let neighbor_row = row as isize + mask_row as isize - 1;
                    let neighbor_col = col as isize + mask_col as isize - 1;
                    neighbors.push(self.get_cell(neighbor_row, neighbor_col).clone());
                }
            }
        }
        
        neighbors
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
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let grid = Grid::new(5, 5, mask);
        assert_eq!(grid.width, 5);
        assert_eq!(grid.height, 5);
        assert_eq!(grid.cells.len(), 5);
        assert_eq!(grid.cells[0].len(), 5);
    }

    #[test]
    fn test_wrapping() {
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let grid = Grid::new(5, 5, mask);
        let cell = grid.get_cell(-1, -1);
        assert_eq!(cell.position, (4, 4));
    }
}
