use crate::grid::Grid;


#[derive(Clone, Debug)]
pub struct Neighborhood {
    pub width: usize,
    pub height: usize,
    pub center_row: usize,
    pub center_col: usize,
    pub mask: Vec<Vec<u8>>,
}

impl Neighborhood {
    pub fn new(
        width: usize,
        height: usize,
        center_row: usize,
        center_col: usize,
        mask: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            width,
            height,
            center_row,
            center_col,
            mask,
        }
    }

    #[inline(always)]
    pub fn get_grid_coords(&self, mask_r: usize, mask_c: usize, cell_r: usize, cell_c: usize, grid: &Grid) -> (usize, usize) {
        let dr = mask_r as isize - self.center_row as isize;
        let dc = mask_c as isize - self.center_col as isize;

        grid.get_position(cell_r as isize + dr, cell_c as isize + dc)
    }

    #[inline(always)]
    pub fn is_valid(&self, mask_r: usize, mask_c: usize) -> u8 {
        unsafe {
            *self.mask.get_unchecked(mask_r).get_unchecked(mask_c)
        }
    }
}