use crate::types::{Instance, ParallelSchedule};
use pathfinding::prelude::{DenseCapacity, EdmondsKarp};
use std::cmp::max;

/// Parallel Left-to-Right Algorithm
pub fn pltr(instance: &Instance) -> ParallelSchedule {
    let (mut nw, _matrix) = create_graph(instance);
    //visualize(&matrix, &nw);
    for k in (1..instance.m + 1).rev() {
        let mut t = 0;
        while t < instance.d_max {
            t = keepidle(k, t, &mut nw, instance);
            //visualize(&matrix, &nw);
            if t < instance.d_max {
                t = keepbusy(k, t, &mut nw, instance);
                //visualize(&matrix, &nw);
            }
        }
    }
    println!("Finished instance");
    ParallelSchedule::from_flow(&nw, instance)
}

/// helper for pltr
fn binary_search_maximum<F>(mut predicate: F, a: usize, b: usize) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let mut a = a;
    let mut b = b;
    let mut t = a + (b - a) / 2;
    //let mut t: usize = (b - a) / 2;
    while b - a > 1 {
        //println!("\t\ta:{}, b:{}, t:{}", a, b, t);
        if predicate(t) {
            a = t;
        } else {
            b = t;
        }
        t = a + (b - a) / 2;
    }
    if predicate(t) {
        Some(t)
    } else {
        None
    }
}

/// helper for pltr
fn capacity(nw: &DenseCapacity<i32>, v1: usize, v2: usize) -> i32 {
    nw.flow(v1, v2) + nw.residual_capacity(v1, v2)
}

/// helper for pltr
fn keepidle_from_to(
    k: usize,
    from: usize,
    to: usize,
    nw: &mut DenseCapacity<i32>,
    instance: &Instance,
) -> bool {
    for t in from..to {
        let v_t = instance.v_node(t);
        let l_t = capacity(nw, v_t, instance.omega_node());
        if l_t >= k as i32 {
            return false;
        }
        let m_t = capacity(nw, v_t, instance.gamma_node()) + l_t;
        assert!(m_t == k as i32);
        nw.set_capacity(v_t, instance.gamma_node(), (k - 1) as i32 - l_t);
        //println!("set_capacity: v{}({})->gamma: {}", t, v_t, (k-1) as i32 - l_t);
    }
    let (_, max_flow, _) = nw.augment();
    //println!("\tmax_flow: {}", max_flow);
    assert!(max_flow <= instance.p_total as i32);
    max_flow == instance.p_total as i32
}

/// helper for pltr
fn keepidle(k: usize, from: usize, nw: &mut DenseCapacity<i32>, instance: &Instance) -> usize {
    println!("\tkeepidle processor {k} from {from} on");
    let can_keepidle = |upto| {
        let mut my_nw = nw.clone();
        keepidle_from_to(k, from, upto, &mut my_nw, instance)
    };
    let upto = binary_search_maximum(can_keepidle, from, instance.d_max + 1)
        .expect("Bounded instance should have remained feasible");

    keepidle_from_to(k, from, upto, nw, instance);
    println!("\tkept idle processor {k} from {from} upto {upto}");
    //println!("{}", ParallelSchedule::from_flow(nw, instance));
    upto
}

/// helper for pltr
fn keepbusy_from_to(
    k: usize,
    from: usize,
    to: usize,
    nw: &mut DenseCapacity<i32>,
    instance: &Instance,
) -> bool {
    let mut total_increase = 0;
    let omega = instance.omega_node();
    let gamma = instance.gamma_node();
    for t in from..to {
        let v_t = instance.v_node(t);
        let old_l_t = capacity(nw, v_t, omega);
        let new_l_t = max(k as i32, old_l_t);
        assert!(new_l_t > 0);
        let increase = new_l_t - old_l_t;
        nw.set_capacity(v_t, omega, new_l_t);
        //println!("set_capacity: v{t}({v_t})->omega: {new_l_t}");
        let new_gamma_cap = capacity(nw, v_t, gamma) - increase;
        assert!(new_gamma_cap >= 0);
        nw.set_capacity(v_t, gamma, new_gamma_cap);

        //println!("set_capacity: v{t}({v_t})->gamma: {new_gamma_cap}");
        total_increase += increase;
    }
    let new_cap_gamma_omega = capacity(nw, gamma, omega) - total_increase;
    if new_cap_gamma_omega < 0 {
        return false;
    }
    nw.set_capacity(gamma, omega, new_cap_gamma_omega);
    //println!("set_capacity: gamma->omega: {new_cap_gamma_omega}");
    let (_, max_flow, _) = nw.augment();
    //println!("\tmax_flow: {}", max_flow);
    assert!(max_flow <= instance.p_total as i32);
    max_flow == instance.p_total as i32
}

/// helper for pltr
fn keepbusy(k: usize, from: usize, nw: &mut DenseCapacity<i32>, instance: &Instance) -> usize {
    println!("\tkeepbusy processor {k} from {from} on");
    let can_keepbusy = |upto| {
        let mut my_nw = nw.clone();
        keepbusy_from_to(k, from, upto, &mut my_nw, instance)
    };

    let upto = binary_search_maximum(can_keepbusy, from, instance.d_max + 1)
        .expect("Bounded instance should have remained feasible");
    keepbusy_from_to(k, from, upto, nw, instance);
    println!("\tkept busy processor {k} from {from} upto {upto}");
    //println!("{}", ParallelSchedule::from_flow(nw, instance));
    upto
}

/// convert a list v representing a n x n matrix into an actual matrix
/// panics if v.len() is not a square number
fn vec_to_square_matrix<T>(v: &[T], n: usize) -> Vec<Vec<T>>
where
    T: Copy,
{
    // TODO: panic if v.len() is not a square number
    let mut m: Vec<Vec<T>> = Vec::new();
    for i in 0..n {
        let mut row: Vec<T> = Vec::new();
        for j in 0..n {
            row.push(v[n * i + j]);
        }
        m.push(row);
    }
    m
}

/// creates the maximum-flow network corresponding to the problem instance
fn create_graph(instance: &Instance) -> (DenseCapacity<i32>, Vec<Vec<i32>>) {
    let mut adj_matrix: Vec<i32> = Vec::new();
    // alpha -- alpha
    adj_matrix.push(0);
    // alpha -> u_j
    for job in instance.jobs.iter() {
        adj_matrix.push(job.p as i32);
    }
    // alpha -- t
    for _t in 0..instance.d_max {
        adj_matrix.push(0);
    }
    // alpha -- gamma
    adj_matrix.push(0);
    // alpha -- omega
    adj_matrix.push(0);

    // u_j (-> v_t)
    for job in instance.jobs.iter() {
        // u_j -- alpha
        adj_matrix.push(0);
        // u_j -- u_j2
        for _job2 in instance.jobs.iter() {
            adj_matrix.push(0);
        }
        // u_j -> v_t
        for t in 0..instance.d_max {
            if job.r <= t && t < job.d {
                adj_matrix.push(1)
            } else {
                adj_matrix.push(0);
            }
        }
        adj_matrix.push(0);
        adj_matrix.push(0);
    }
    // v_t (-> gamma, omega)
    for _t in 0..instance.d_max {
        adj_matrix.push(0);
        for _job in instance.jobs.iter() {
            adj_matrix.push(0);
        }
        for _t2 in 0..instance.d_max {
            adj_matrix.push(0);
        }
        let m = instance.m;
        let l = 0;
        adj_matrix.push(m as i32);
        adj_matrix.push(l);
    }
    // gamma
    for _node in 0..1 + instance.jobs.len() + instance.d_max + 1 {
        adj_matrix.push(0);
    }
    let sum_l_t: i32 = 0;
    adj_matrix.push((instance.p_total as i32) - sum_l_t);
    // omega
    for _node in 0..1 + instance.jobs.len() + instance.d_max + 2 {
        adj_matrix.push(0);
    }
    let alpha = 0;
    let omega = instance.jobs.len() + instance.d_max + 2;
    let my_matrix = vec_to_square_matrix(&adj_matrix, omega + 1);
    let nw = DenseCapacity::from_vec(alpha, omega, adj_matrix);
    (nw, my_matrix)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Job;
    #[test]
    fn bs_on_integers() {
        let v: Vec<usize> = (0..10).collect();
        let threshold = 5;
        let is_small = |i| v[i] < threshold;
        let res = binary_search_maximum(is_small, 0, 10);
        assert_eq!(res, Some(4));
    }
    #[test]
    fn bs_all_fulfill_pred() {
        let v: Vec<usize> = (0..10).collect();
        let threshold = 11;
        let is_small = |i| v[i] < threshold;
        let res = binary_search_maximum(is_small, 0, 10);
        assert_eq!(res, Some(9));
    }
    #[test]
    fn bs_none_fulfill_pred() {
        let v: Vec<i32> = (0..10).collect();
        let threshold = 5;
        let is_small = |i| v[i] < threshold;
        let res = binary_search_maximum(is_small, 5, 10);
        assert_eq!(res, None);
    }
    #[test]
    fn bs_range_limits() {
        let v: Vec<usize> = (0..10).collect();
        let threshold = 11;
        let is_small = |i| v[i] < threshold;
        let res = binary_search_maximum(is_small, 0, 7);
        assert_eq!(res, Some(6));
    }

    #[test]
    fn simple_unit_instance() {
        let jobs: Vec<Job> = (0..10)
            .map(|i| Job {
                id: i,
                r: i,
                d: i + 1,
                p: 1,
            })
            .collect();
        let m = 1;
        let q = 1;
        let instance = Instance::new(jobs, m, q);
        let schedule = pltr(&instance);
        let target_schedule: ParallelSchedule =
            ParallelSchedule::from_vec((0..10).map(|i| vec![i]).collect(), m);
        assert!(target_schedule.is_valid_for(&instance).is_ok());
        assert_eq!(target_schedule, schedule);
    }
    #[test]
    fn simple_on_off() {
        let jobs: Vec<Job> = (0..10)
            .map(|i| Job {
                id: i,
                r: 2 * i,
                d: 2 * i + 1,
                p: 1,
            })
            .collect();
        let m = 2;
        let q = 1;
        let instance = Instance::new(jobs, m, q);
        let schedule = pltr(&instance);
        let target_schedule_vec = (0..19)
            .map(|tslot| {
                if tslot % 2 == 0 {
                    let id: usize = tslot / 2;
                    vec![id]
                } else {
                    vec![]
                }
            })
            .collect();
        let target_schedule = ParallelSchedule::from_vec(target_schedule_vec, m);
        assert!(target_schedule.is_valid_for(&instance).is_ok());
        assert_eq!(target_schedule, schedule);
    }
    #[test]
    fn simultaneous_on_off() {
        // different jobs get the same id here
        let jobs: Vec<Job> = (0..10)
            .flat_map(|i| {
                vec![
                    Job {
                        id: 4 * i,
                        r: 2 * i,
                        d: 2 * i + 1,
                        p: 1,
                    },
                    Job {
                        id: 4 * i + 1,
                        r: 2 * i,
                        d: 2 * i + 1,
                        p: 1,
                    },
                    Job {
                        id: 4 * i + 2,
                        r: 2 * i,
                        d: 2 * i + 1,
                        p: 1,
                    },
                    Job {
                        id: 4 * i + 3,
                        r: 2 * i,
                        d: 2 * i + 1,
                        p: 1,
                    },
                ]
            })
            .collect();
        let m = 5;
        let q = 1;
        let instance = Instance::new(jobs, m, q);
        let schedule = pltr(&instance);
        let target_schedule_vec = (0..19)
            .map(|tslot| {
                if tslot % 2 == 0 {
                    let id: usize = tslot * 2;
                    vec![id, id + 1, id + 2, id + 3]
                } else {
                    vec![]
                }
            })
            .collect();
        let target_schedule = ParallelSchedule::from_vec(target_schedule_vec, m);
        assert!(target_schedule.is_valid_for(&instance).is_ok());
        assert_eq!(target_schedule, schedule);
    }

    #[test]
    fn test_around_d_max() {
        let mut jobs: Vec<Job> = (0..10)
            .map(|i| Job {
                id: i,
                r: i,
                d: i + 1,
                p: 1,
            })
            .collect();
        jobs.push(Job {
            id: 10,
            r: 6,
            d: 10,
            p: 2,
        });
        let m = 2;
        let q = 1;
        let instance = Instance::new(jobs, m, q);
        let mut target_schedule_vec: Vec<Vec<usize>> = (0..10).map(|i| vec![i]).collect();
        target_schedule_vec[8].push(10);
        target_schedule_vec[9].push(10);
        let target_schedule: ParallelSchedule = ParallelSchedule::from_vec(target_schedule_vec, m);
        assert!(target_schedule.is_valid_for(&instance).is_ok());
        let schedule = pltr(&instance);
        assert_eq!(target_schedule, schedule);
    }

    #[test]
    fn generated_instance() {
        let instance = Instance {
            jobs: vec![
                Job {
                    id: 3,
                    r: 5,
                    d: 12,
                    p: 3,
                },
                Job {
                    id: 2,
                    r: 12,
                    d: 13,
                    p: 1,
                },
                Job {
                    id: 4,
                    r: 12,
                    d: 13,
                    p: 1,
                },
                Job {
                    id: 1,
                    r: 9,
                    d: 14,
                    p: 1,
                },
                Job {
                    id: 7,
                    r: 7,
                    d: 14,
                    p: 1,
                },
                Job {
                    id: 0,
                    r: 10,
                    d: 17,
                    p: 1,
                },
                Job {
                    id: 6,
                    r: 16,
                    d: 17,
                    p: 1,
                },
                Job {
                    id: 5,
                    r: 15,
                    d: 18,
                    p: 1,
                },
            ],
            m: 6,
            q: 1,
            d_max: 18,
            p_total: 10,
        };
        let _schedule = pltr(&instance);
    }

    #[test]
    fn generated_instance_two() {
        let instance = Instance {
            jobs: vec![
                Job {
                    id: 9,
                    r: 2,
                    d: 3,
                    p: 1,
                },
                Job {
                    id: 5,
                    r: 3,
                    d: 5,
                    p: 1,
                },
                Job {
                    id: 0,
                    r: 4,
                    d: 7,
                    p: 1,
                },
                Job {
                    id: 7,
                    r: 4,
                    d: 8,
                    p: 2,
                },
                Job {
                    id: 1,
                    r: 5,
                    d: 9,
                    p: 2,
                },
                Job {
                    id: 2,
                    r: 8,
                    d: 9,
                    p: 1,
                },
                Job {
                    id: 3,
                    r: 3,
                    d: 9,
                    p: 3,
                },
                Job {
                    id: 4,
                    r: 8,
                    d: 11,
                    p: 1,
                },
                Job {
                    id: 6,
                    r: 4,
                    d: 11,
                    p: 2,
                },
                Job {
                    id: 8,
                    r: 6,
                    d: 11,
                    p: 2,
                },
            ],
            m: 5,
            q: 1,
            d_max: 11,
            p_total: 16,
        };
        let _schedule = pltr(&instance);
    }
    #[test]
    fn generated_valley_instance_mid() {
        let instance = Instance {
            jobs: vec![
                Job {
                    id: 1,
                    r: 4,
                    d: 5,
                    p: 1,
                },
                Job {
                    id: 7,
                    r: 4,
                    d: 5,
                    p: 1,
                },
                Job {
                    id: 3,
                    r: 1,
                    d: 10,
                    p: 2,
                },
                Job {
                    id: 5,
                    r: 3,
                    d: 10,
                    p: 2,
                },
                Job {
                    id: 0,
                    r: 18,
                    d: 20,
                    p: 1,
                },
                Job {
                    id: 6,
                    r: 16,
                    d: 24,
                    p: 1,
                },
                Job {
                    id: 4,
                    r: 25,
                    d: 28,
                    p: 1,
                },
                Job {
                    id: 2,
                    r: 22,
                    d: 29,
                    p: 1,
                },
            ],
            m: 5,
            q: 1,
            d_max: 29,
            p_total: 10,
        };
        let _schedule = pltr(&instance);
    }

    #[test]
    fn generated_valley_instance_small() {
        let instance = Instance {
            jobs: vec![
                Job {
                    id: 0,
                    r: 3,
                    d: 4,
                    p: 1,
                },
                Job {
                    id: 4,
                    r: 3,
                    d: 4,
                    p: 1,
                },
                Job {
                    id: 3,
                    r: 1,
                    d: 6,
                    p: 2,
                },
                Job {
                    id: 1,
                    r: 7,
                    d: 8,
                    p: 1,
                },
                Job {
                    id: 2,
                    r: 4,
                    d: 9,
                    p: 2,
                },
            ],
            m: 5,
            q: 1,
            d_max: 9,
            p_total: 7,
        };
        let _schedule = pltr(&instance);
    }
}
