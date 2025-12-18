use crate::cell::Cell;
use crate::grid::Grid;

/// A read-only view of cells around a center position
pub struct Neighborhood<'a> {
    pub width: usize,
    pub height: usize,
    pub center_row: usize,
    pub center_col: usize,
    pub row: usize,
    pub col: usize,
    pub mask: &'a [Vec<bool>],
    pub cells: Vec<Vec<&'a Cell>>,
}

impl<'a> Neighborhood<'a> {
    /// Create a new neighborhood view for a specific grid position
    pub fn new(width: usize,
               height: usize,
               center_row: usize,
               center_col: usize,
               row: usize,
               col: usize,
               mask: &'a [Vec<bool>],
               grid: &'a Grid,
            ) -> Self {

        let mut cells = Vec::with_capacity(height);

        for delta_row in 0..height {
            let mut row_cells = Vec::with_capacity(width);

            for delta_col in 0..width {
                if mask[delta_row][delta_col] {
                    let nr = row as isize + delta_row as isize - center_row as isize;
                    let nc = col as isize + delta_col as isize - center_col as isize;
                    row_cells.push(grid.get_cell(nr, nc));
                } else {
                    row_cells.push(Cell::empty());
                }
            }

            cells.push(row_cells);
        }

        Self {
            width,
            height,
            center_row,
            center_col,
            row,
            col,
            mask,
            cells,
        }
    }

    /// Create a new neighborhood using an existing neighborhood as a template
    #[inline]
    pub fn new_from_base(row: usize,
                         col: usize,
                         base: &Neighborhood<'a>,
                         grid: &'a Grid,
                        ) -> Self {

        Self::new(
            base.width,
            base.height,
            base.center_row,
            base.center_col,
            row,
            col,
            base.mask,
            grid,
        )
    }

    /// Refresh neighborhood cells after grid changes
    pub fn update_cells(&mut self, grid: &'a Grid) {
        for delta_row in 0..self.height {
            for delta_col in 0..self.width {
                if self.mask[delta_row][delta_col] {
                    let nr = self.row as isize + delta_row as isize - self.center_row as isize;
                    let nc = self.col as isize + delta_col as isize - self.center_col as isize;
                    self.cells[delta_row][delta_col] = grid.get_cell(nr, nc);
                }
            }
        }
    }

    /// Update mask and refresh neighborhood cells
    pub fn update_mask(&mut self, mask: &'a [Vec<bool>], grid: &'a Grid) {
        self.mask = mask;
        self.update_cells(grid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighborhood_creation() {
        let grid = Grid::new(5, 5);
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];

        let neighborhood = Neighborhood::new(3, 3, 1, 1, 2, 2, &mask, &grid);

        assert_eq!(neighborhood.width, 3);
        assert_eq!(neighborhood.height, 3);
        assert_eq!(neighborhood.center_row, 1);
        assert_eq!(neighborhood.center_col, 1);
    }
}
