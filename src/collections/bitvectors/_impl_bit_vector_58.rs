use crate::collections::bitvectors::{BitVector, BitVector58};
use std::fmt::{Debug, Display, Formatter};

impl BitVector58 {
    /// **(internal)** Check if the given index is valid in this `BitVector` - panic otherwise.
    /// Only enabled when `shields_up` is set.
    fn check_access(&self, index: usize) {
        if cfg!(shields_up) && index >= self.len() {
            panic!(
                "Accessing element {} in a BitVector of length {}.",
                index,
                self.len()
            );
        }
    }
}

impl BitVector for BitVector58 {
    fn empty(len: usize) -> Self {
        if len > 58 {
            panic!("This implementation of BitVector supports only up-to 58 entries");
        }
        return BitVector58((len << 58) as u64);
    }

    fn len(&self) -> usize {
        return (self.0 >> 58) as usize;
    }

    fn get(&self, index: usize) -> bool {
        self.check_access(index);
        return self.0 & ((1 << index) as u64) != 0;
    }

    fn set(&mut self, index: usize, value: bool) {
        self.check_access(index);
        if value {
            self.0 |= (1 << index) as u64;
        } else {
            self.0 &= !(1 << index) as u64;
        }
    }

    fn flip(&mut self, index: usize) {
        self.check_access(index);
        self.0 ^= (1 << index) as u64;
    }
}

impl Display for BitVector58 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return self.display(f);
    }
}

impl From<Vec<bool>> for BitVector58 {
    fn from(data: Vec<bool>) -> Self {
        return Self::from_bool_vector(data);
    }
}

impl Debug for BitVector58 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "BitVector58({})[", self.len())?;
        for i in 0..self.len() {
            write!(f, "{}", if self.get(i) { 1 } else { 0 })?;
        }
        write!(f, "]")?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::bitvectors::{BitVector, BitVector58};

    #[test]
    fn test_array_bit_vector() {
        let mut bv = BitVector58::empty(10);
        assert_eq!(vec![false; 10], bv.values());
        bv.set(2, true);
        bv.flip(6);
        assert!(bv.get(2));
        assert!(bv.get(6));
        assert_eq!(vec![2, 6], bv.ones());
        assert_eq!(vec![0, 1, 3, 4, 5, 7, 8, 9], bv.zeros());
        assert_eq!(bv, BitVector58::from_ones(10, vec![2, 6]));
        assert_eq!(
            bv,
            BitVector58::from(vec![
                false, false, true, false, false, false, true, false, false, false
            ])
        );
        println!("{:?} is displayed as {}", bv, bv);
        bv.set(6, false);
        assert!(!bv.get(6));
        bv.flip(2);
        assert!(!bv.get(2));
    }
}
