/*use crate::graph::EvolutionOperator;
use crate::lock_free_array::LockFreeArray;
use crate::lock_free_array_queue::LockFreeArrayQueue;
use crate::parameters::ParamSet;
use crossbeam::scope;

pub fn reachability<P, G>(graph: G, initial: &Vec<P>, parallelism: usize, empty: P) -> Vec<P>
where
    P: ParamSet,
    G: EvolutionOperator<P>,
    G: Sync,
{
    let result = LockFreeArray::new(initial.len(), empty);
    let queue = LockFreeArrayQueue::new(initial.len());
    for i in 0..initial.len() {
        if !initial[i].is_empty() {
            queue.set(i);
            result.update(i, |val| {
                *val = initial[i].clone();
            });
        }
    }
    scope(|s| {
        for _ in 0..parallelism {
            s.spawn(|_| {
                let mut work_in_progress = true;
                while work_in_progress {
                    work_in_progress = false;
                    let mut next_state = 0;
                    while let Some(next) = queue.next(next_state) {
                        next_state = next + 1;
                        for (successor, edge_params) in graph.step(next) {
                            let transfer_params = result.get(next).intersect(&edge_params);
                            let current = result.get(successor);
                            if transfer_params.is_subset_of(current) {
                                continue;
                            }
                            let update = result.update(successor, |value| {
                                let new_value = value.union(&transfer_params);
                                let is_new = new_value.is_subset_of(value);
                                *value = new_value;
                                is_new
                            });
                            match update {
                                None => {
                                    // Busy... reinsert into queue and continue
                                    queue.set(next);
                                    work_in_progress = true;
                                }
                                Some(false) => {
                                    // Do nothing... update was successful, but nothing has changed
                                }
                                Some(true) => {
                                    // Update was a success and it changed something - add the successor
                                    queue.set(successor);
                                    work_in_progress = true;
                                }
                            }
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let mut actual_result = Vec::with_capacity(initial.len());
    for i in 0..initial.len() {
        actual_result.push(result.get(i).clone());
    }
    return actual_result;
}
*/
