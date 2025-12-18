use rand::Rng;

/// A cell with a 9-trait fingerprint
#[derive(Clone, Debug)]
pub struct Cell {
    pub fingerprint: [f32; 9],
    pub position: (usize, usize),
}

impl Cell {
    /// Create a new cell with all traits initialized to 0.0
    pub fn new(position: (usize, usize)) -> Self {
        Self {
            fingerprint: [0.0; 9],
            position,
        }
    }

    /// Create a cell with random trait values
    pub fn random(position: (usize, usize)) -> Self {
        let mut rng = rand::thread_rng();
        let mut fingerprint = [0.0; 9];
        for trait_val in fingerprint.iter_mut() {
            *trait_val = rng.gen_range(0.0..=1.0);
        }
        Self {
            fingerprint,
            position,
        }
    }

    /// Get a specific trait value
    pub fn get_trait(&self, index: usize) -> f32 {
        self.fingerprint[index]
    }

    /// Set a specific trait value (clamped to [0.0, 1.0])
    pub fn set_trait(&mut self, index: usize, value: f32) {
        self.fingerprint[index] = value.clamp(0.0, 1.0);
    }

    /// Get the trait index from 2D grid coordinates (row, col)
    pub fn trait_index(row: usize, col: usize) -> usize {
        row * 3 + col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_index() {
        assert_eq!(Cell::trait_index(0, 0), 0);
        assert_eq!(Cell::trait_index(0, 2), 2);
        assert_eq!(Cell::trait_index(1, 1), 4);
        assert_eq!(Cell::trait_index(2, 2), 8);
    }

    #[test]
    fn test_clamp() {
        let mut cell = Cell::new((0, 0));
        cell.set_trait(0, 1.5);
        assert_eq!(cell.get_trait(0), 1.0);
        cell.set_trait(0, -0.5);
        assert_eq!(cell.get_trait(0), 0.0);
    }
}
