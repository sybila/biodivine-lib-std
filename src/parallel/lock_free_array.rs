/*use std::mem::forget;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct LockFreeArray<I>
where
    I: Clone,
{
    capacity: usize,
    items: *mut I,
    busy: Vec<AtomicBool>,
}

impl<I> LockFreeArray<I>
where
    I: Clone,
{
    pub fn new(capacity: usize, default: I) -> LockFreeArray<I> {
        let mut items = Vec::with_capacity(capacity);
        let mut busy = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            items.push(default.clone());
            busy.push(AtomicBool::new(false));
        }
        let result = LockFreeArray {
            capacity,
            busy,
            items: items.as_mut_ptr(),
        };
        forget(items);
        return result;
    }

    pub fn update<F, R>(&self, position: usize, update: F) -> Option<R>
    where
        F: FnOnce(&mut I) -> R,
    {
        self.check_position(position);
        let lock = &self.busy[position];
        return if lock.compare_and_swap(false, true, Ordering::SeqCst) == false {
            // lock acquired
            let item = unsafe { &mut *self.items.add(position) };
            let result = update(item);
            lock.store(false, Ordering::SeqCst);
            Some(result)
        } else {
            // resource busy
            None
        };
    }

    // TODO: This is not correct!
    pub fn get(&self, position: usize) -> &I {
        self.check_position(position);
        return unsafe { &*self.items.add(position) };
    }

    fn check_position(&self, position: usize) {
        if position >= self.capacity {
            panic!("Position {} >= {} out of bounds", position, self.capacity);
        }
    }
}

unsafe impl<I> Send for LockFreeArray<I> where I: Clone {}
unsafe impl<I> Sync for LockFreeArray<I> where I: Clone {}

impl<I> Drop for LockFreeArray<I>
where
    I: Clone,
{
    fn drop(&mut self) {
        drop(self.items);
    }
}

#[cfg(test)]
mod tests {
    use super::LockFreeArray;
    use crossbeam::thread::scope;

    #[test]
    fn single_thread_update_test() {
        let array = LockFreeArray::new(10, 0 as usize);
        for i in 0..10 {
            assert_eq!(
                Some((9 - i) * 2),
                array.update(i, |v| {
                    *v = 9 - i;
                    return (*v) * 2;
                })
            );
        }
        for i in 0..10 {
            assert_eq!(9 - i, *array.get(i))
        }
    }

    /// Each thread counts to 10_000 in its respective array cell.
    /// Since threads are not fighting, every update should succeed.
    #[test]
    fn multi_thread_update_test_no_contention() {
        let array = LockFreeArray::new(10, 0 as usize);
        scope(|s| {
            for id in 0..10 {
                let array = &array;
                s.spawn(move |_| {
                    for i in 0..10_000 {
                        assert_eq!(
                            Some(i + 1),
                            array.update(id, |val| {
                                *val = (*val) + 1;
                                *val
                            })
                        )
                    }
                });
            }
            // scope will join the threads for us
        })
        .unwrap();
        for i in 0..10 {
            assert_eq!(10_000, *array.get(i));
        }
    }

    /// Each thread counts to 10_000 inside one specific cell.
    /// Since threads are constantly fighting, the updates can fail. however,
    /// in the end, the value we counted to should be correct.
    #[test]
    fn multi_thread_update_with_contention() {
        let array = LockFreeArray::new(10, 0 as usize);
        scope(|s| {
            for _ in 0..10 {
                let array = &array;
                s.spawn(move |_| {
                    let mut count = 0;
                    while count < 10_000 {
                        let update = array.update(3, |val| *val = *val + 1);
                        if Some(()) == update {
                            count += 1;
                        }
                    }
                });
            }
        })
        .unwrap();
        assert_eq!(10_000 * 10, *array.get(3));
    }
}
*/
