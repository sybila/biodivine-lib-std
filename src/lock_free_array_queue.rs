use std::sync::atomic::{AtomicBool, Ordering};

pub struct LockFreeArrayQueue {
    items: Vec<AtomicBool>,
}

unsafe impl Send for LockFreeArrayQueue {}
unsafe impl Sync for LockFreeArrayQueue {}

impl LockFreeArrayQueue {
    pub fn new(capacity: usize) -> LockFreeArrayQueue {
        let mut items = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            items.push(AtomicBool::new(false));
        }
        return LockFreeArrayQueue { items };
    }

    /// Set value at `position` to true, returning true if the value has been added
    /// and false if the value was already present.
    pub fn set(&self, position: usize) -> bool {
        let item = &self.items[position];
        return !item.swap(true, Ordering::SeqCst);
    }

    /// Find the next set value in this queue starting at the given index.
    /// If no value is set after `starting_at`, return None.
    pub fn next(&self, starting_at: usize) -> Option<usize> {
        for i in starting_at..self.items.len() {
            let item = &self.items[i];
            if item.compare_and_swap(true, false, Ordering::SeqCst) == true {
                return Some(i);
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::lock_free_array_queue::LockFreeArrayQueue;
    use crossbeam::scope;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn single_thread_test() {
        let queue = LockFreeArrayQueue::new(10);
        assert_eq!(None, queue.next(0));
        queue.set(3);
        assert_eq!(None, queue.next(10));
        assert_eq!(None, queue.next(4));
        assert_eq!(Some(3), queue.next(0));
        queue.set(3);
        queue.set(7);
        queue.set(8);
        queue.set(7);
        assert_eq!(Some(7), queue.next(5));
        assert_eq!(Some(3), queue.next(3));
        assert_eq!(Some(8), queue.next(3));
        assert_eq!(None, queue.next(0));
    }

    /// Each thread inserts values at pseudo random order and then extracts them.
    /// We count how many unique inserts were successful at every position and how
    /// many were actually extracted. This way, we should end up with a zero in
    /// the end.
    #[test]
    fn multi_thread_test() {
        let total_actual_ops = AtomicU32::new(0);
        let mut counts: Vec<AtomicU32> = Vec::new();
        for _ in 0..10 {
            counts.push(AtomicU32::new(0));
        }
        let queue = LockFreeArrayQueue::new(10);
        scope(|s| {
            for id in 0..10 {
                let total_actual_ops = &total_actual_ops;
                let counts = &counts;
                let queue = &queue;
                s.spawn(move |_| {
                    for _ in 0..10_000 {
                        for i in 0..10 {
                            let position = (i + id) % 10;
                            if queue.set(position) {
                                let counter = &counts[position];
                                counter.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                        let mut next = 0;
                        while let Some(found) = queue.next(next) {
                            total_actual_ops.fetch_add(1, Ordering::SeqCst);
                            let counter = &counts[found];
                            counter.fetch_sub(1, Ordering::SeqCst);
                            next = found;
                        }
                    }
                });
            }
        })
        .unwrap();

        for i in 0..10 {
            assert_eq!(0, counts[i].load(Ordering::SeqCst));
        }
        assert!(total_actual_ops.load(Ordering::SeqCst) > 0);
    }
}
