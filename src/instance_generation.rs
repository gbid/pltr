use crate::scheduling;
use crate::types::{Instance, Job};
use rand::Rng;
use std::cmp;
//problem instance generation
fn generate_random_jobs(n: usize, upto: usize, interval_avg: usize) -> Vec<Job> {
    let mut jobs: Vec<Job> = Vec::with_capacity(n);
    let mut rng = rand::thread_rng();
    for i in 0..n {
        let interval_size: usize = rng.gen_range(1..cmp::max(2, 2 * interval_avg));
        assert!(upto - interval_size > 1);
        let r: usize = rng.gen_range(0..upto - interval_size);
        let d: usize = r + interval_size;
        let p: usize = rng.gen_range(1..=cmp::max(1, interval_size / 2));
        let job = Job { id: i, r, d, p };
        jobs.push(job);
    }
    jobs
}
pub fn generate_random_instance(
    n: usize,
    upto: usize,
    interval_avg: usize,
    q: usize,
    m: usize,
) -> Instance {
    let mut jobs = generate_random_jobs(n, upto, interval_avg);
    jobs.sort_by(|job1, job2| job1.d.cmp(&job2.d));
    Instance::new(jobs, m, q)
}

pub fn generate_valley_instance(
    n: usize,
    upto: usize,
    interval_avg: usize,
    q: usize,
    m: usize,
    valley_num: usize,
) -> Instance {
    let valley_size = upto / valley_num;
    let mut jobs = generate_random_jobs(n, valley_size, interval_avg);
    for (i, job) in jobs.iter_mut().enumerate() {
        let offset = valley_size * (i % valley_num);
        job.r += offset;
        job.d += offset;
    }
    jobs.sort_by(|job1, job2| job1.d.cmp(&job2.d));
    Instance::new(jobs, m, q)
}

pub fn generate_small_deterministic_problem_instance() -> Instance {
    let mut jobs = vec![
        Job {
            id: 1,
            r: 1,
            d: 3,
            p: 1,
        },
        Job {
            id: 2,
            r: 1,
            d: 10,
            p: 2,
        },
        Job {
            id: 3,
            r: 6,
            d: 7,
            p: 1,
        },
        Job {
            id: 4,
            r: 7,
            d: 9,
            p: 2,
        },
    ];
    jobs.sort_by(|job1, job2| job1.d.cmp(&job2.d));
    let d_max = scheduling::get_d_max(&jobs) + 1;
    let p_total = scheduling::get_total_processing_volume(&jobs);
    Instance {
        jobs,
        q: 1,
        m: 1,
        d_max,
        p_total,
    }
}
