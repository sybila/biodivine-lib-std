use super::{IdState, State};
use std::fmt::{Display, Error, Formatter};

impl State for IdState {}

impl From<usize> for IdState {
    fn from(val: usize) -> Self {
        return IdState(val);
    }
}

impl Into<usize> for IdState {
    fn into(self) -> usize {
        return self.0;
    }
}

impl Display for IdState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        return write!(f, "State({})", self.0);
    }
}

impl IdState {
    /// Test if the bit at the given position is set or not.
    pub fn get_bit(self, bit: usize) -> bool {
        return (self.0 >> bit) & 1 == 1;
    }

    /// Flip the bit a the given position.
    pub fn flip_bit(self, bit: usize) -> IdState {
        return IdState(self.0 ^ (1 << bit));
    }
}

#[cfg(test)]
mod tests {
    use crate::IdState;

    #[test]
    fn id_state_test() {
        let state = IdState::from(0b10110);
        assert!(!state.get_bit(0));
        assert!(state.get_bit(1));
        assert!(state.get_bit(2));
        assert!(!state.get_bit(3));
        assert!(state.get_bit(4));
        let flipped = state.flip_bit(3);
        assert_eq!(0b11110 as usize, flipped.into());
    }

}
