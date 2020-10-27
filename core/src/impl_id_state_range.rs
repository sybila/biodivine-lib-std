use super::{IdState, IdStateRange};

impl IdStateRange {
    pub fn new(state_count: usize) -> IdStateRange {
        return IdStateRange {
            next: 0,
            remaining: state_count,
        };
    }
}

impl Iterator for IdStateRange {
    type Item = IdState;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.remaining == 0 {
            None
        } else {
            let result = self.next;
            self.remaining -= 1;
            self.next += 1;
            Some(IdState::from(result))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{IdState, IdStateRange};

    #[test]
    fn test_state_range_iterator() {
        let mut iter = IdStateRange::new(6);
        assert_eq!(Some(IdState::from(0)), iter.next());
        assert_eq!(Some(IdState::from(1)), iter.next());
        assert_eq!(Some(IdState::from(2)), iter.next());
        assert_eq!(Some(IdState::from(3)), iter.next());
        assert_eq!(Some(IdState::from(4)), iter.next());
        assert_eq!(Some(IdState::from(5)), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }
}
