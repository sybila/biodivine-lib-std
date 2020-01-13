use crate::graph::StateId;
use crate::RangeStateIterator;

impl RangeStateIterator {
    pub fn new(state_count: usize) -> RangeStateIterator {
        return RangeStateIterator {
            next: 0,
            remaining: state_count,
        };
    }
}

impl Iterator for RangeStateIterator {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.remaining == 0 {
            None
        } else {
            let result = self.next;
            self.remaining -= 1;
            self.next += 1;
            Some(StateId(result))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::StateId;
    use crate::RangeStateIterator;

    #[test]
    fn test_state_range_iterator() {
        let mut iter = RangeStateIterator::new(6);
        assert_eq!(Some(StateId(0)), iter.next());
        assert_eq!(Some(StateId(1)), iter.next());
        assert_eq!(Some(StateId(2)), iter.next());
        assert_eq!(Some(StateId(3)), iter.next());
        assert_eq!(Some(StateId(4)), iter.next());
        assert_eq!(Some(StateId(5)), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }
}
