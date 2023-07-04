use pathfinding::prelude::{DenseCapacity, EdmondsKarp};
use std::fmt;
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct Job {
    pub id: usize, //identifier
    pub r: usize,  //release time
    pub d: usize,  // deadline
    pub p: usize,  // processing volume
}
impl Job {
    /// initializes a new job with incremented id
    pub fn new(r: usize, d: usize, p: usize) -> Job {
        /// This is a static variable, meaning it maintains its state across multiple calls to this function
        static mut NEXT_ID: usize = 0;

        // The unsafe block is required because we are accessing a mutable static variable
        // this function is not being accessed from multiple threads
        unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            Job { id, r, d, p }
        }
    }
}

/// single-processor schedule
pub struct Schedule(pub Vec<Option<usize>>);
/// multi-processor schedule
#[derive(PartialEq, Debug)]
pub struct ParallelSchedule(Vec<Vec<usize>>, usize);
impl ParallelSchedule {
    fn timeslots_of(&self, job: &Job) -> Vec<usize> {
        let mut timeslots: Vec<usize> = Vec::with_capacity(job.p);
        for (t, jobs) in self.0.iter().enumerate() {
            for scheduled_job in jobs.iter() {
                if job.id == *scheduled_job {
                    timeslots.push(t);
                }
            }
        }
        timeslots
    }
    pub fn is_valid_for(&self, instance: &Instance) -> Result<(), String> {
        for job in instance.jobs.iter() {
            let slots: Vec<usize> = self.timeslots_of(job);
            if slots.len() != job.p {
                return Err(format!(
                    "job {} not feasibly scheduled: for {} of {} units scheduled",
                    job.id,
                    slots.len(),
                    job.p
                ));
            }
            for t in slots {
                if t < job.r || t >= job.d {
                    return Err(format!("job {} not feasibly scheduled", job.id));
                }
            }
        }
        Ok(())
    }
    pub fn from_flow(nw: &DenseCapacity<i32>, instance: &Instance) -> ParallelSchedule {
        let mut schedule: ParallelSchedule =
            ParallelSchedule(vec![Vec::new(); instance.d_max], instance.m);
        for t in 0..instance.d_max {
            for (j, job) in instance.jobs.iter().enumerate() {
                let v_t = instance.v_node(t);
                let u_j = instance.u_node(j);
                if nw.flow(u_j, v_t) == 1 {
                    schedule.0[t].push(job.id);
                }
            }
        }
        let validation_result = schedule.is_valid_for(instance);
        assert_eq!(
            validation_result,
            Ok(()),
            "\nThe invalid schedule:\n{}",
            schedule
        );
        schedule
    }
    pub fn from_vec(vec: Vec<Vec<usize>>, m: usize) -> ParallelSchedule {
        ParallelSchedule(vec, m)
    }
}

/// all data specifiying a problem instance,
/// in addition to precomputed maximum deadline (d_max) and total processing volume across all jobs (p_total)
#[derive(Clone, Debug)]
pub struct Instance {
    pub jobs: Vec<Job>,
    pub m: usize,
    pub q: usize,
    pub d_max: usize,
    pub p_total: usize,
}


fn get_d_max(jobs: &[Job]) -> usize {
    let mut d_max: usize = 0;
    for j in jobs {
        if j.d > d_max {
            d_max = j.d;
        }
    }
    d_max
}

fn get_total_processing_volume(jobs: &[Job]) -> usize {
    let mut p_total = 0;
    for job in jobs {
        p_total += job.p;
    }
    p_total
}

impl Instance {
    pub fn new(jobs: Vec<Job>, m: usize, q: usize) -> Instance {
        let d_max = get_d_max(&jobs);
        let p_total = get_total_processing_volume(&jobs);
        Instance {
            jobs,
            m,
            q,
            d_max,
            p_total,
        }
    }
    /// returns (the number of) the node u_j in the maximum flow network corresponding to job j
    pub fn u_node(&self, job: usize) -> usize {
        1 + job
    }
    /// returns (the number of) the node v_t in the maximum flow network corresponding to time slot t
    pub fn v_node(&self, tslot: usize) -> usize {
        1 + self.jobs.len() + tslot
    }
    /// returns (the number of) the source node alpha of the flow network
    pub fn alpha_node(&self) -> usize {
        0
    }
    /// returns (the number of) the special node gamma of the flow network which ensures that lower
    /// processor bounds are met
    pub fn gamma_node(&self) -> usize {
        1 + self.jobs.len() + self.d_max
    }
    /// returns (the number of) the sink node omega of the flow network
    pub fn omega_node(&self) -> usize {
        self.gamma_node() + 1
    }
}

// pretty printing
impl fmt::Display for ParallelSchedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines: Vec<String> = Vec::new();

        let mut line = String::new();
        write!(&mut line, "|")?;
        for t in 0..self.0.len() {
            write!(&mut line, "{:03}|", t)?;
        }
        lines.push(line);

        let mut line = String::new();
        write!(&mut line, "|")?;
        for _t in 0..self.0.len() {
            write!(&mut line, "   |")?;
        }
        lines.push(line);

        let slot_with_most_jobs: Option<(usize, _)> = self
            .0
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.len().cmp(&b.len()));
        let max_num_of_jobs: usize = slot_with_most_jobs
            .map(|(t, _)| self.0[t].len())
            .unwrap_or(0);

        for k in 0..max_num_of_jobs {
            let mut line = String::new();
            write!(&mut line, "|")?;
            for t in 0..self.0.len() {
                match self.0[t].get(k) {
                    None => {
                        write!(&mut line, "---|")?;
                    }
                    Some(job_id) => {
                        write!(&mut line, "{:03}|", job_id)?;
                    }
                }
            }
            lines.push(line);
        }

        for _ in max_num_of_jobs..self.1 {
            let mut line = String::new();
            write!(&mut line, "|")?;
            for _ in 0..self.0.len() {
                write!(&mut line, "---|")?;
            }
            lines.push(line);
        }

        // Reverse the order of the lines
        lines.reverse();

        // Write the reversed lines to the output stream
        for line in lines {
            write!(f, "\n{}", line)?;
        }

        Ok(())
    }
}
impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|")?;
        for t in 0..self.0.len() {
            write!(f, "{:03}|", t)?;
        }
        write!(f, "\n|")?;
        for t in 0..self.0.len() {
            match self.0[t] {
                None => {
                    write!(f, "---|")?;
                }
                Some(job_id) => {
                    write!(f, "{:03}|", job_id)?;
                }
            }
        }
        Ok(())
    }
}
impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", " ".repeat(4 * self.r))?;
        if self.d - self.r > 1 {
            write!(f, "[---")?;
            write!(f, "{}", "|---".repeat(self.d - self.r - 2))?;
            write!(f, "|---]")?;
        } else {
            write!(f, "[---]")?;
        }
        write!(f, "p:{}, id:{}", self.p, self.id)?;
        Ok(())
    }
}
impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "d_max: {}, m: {}, p_total: {}, q: {}",
            self.d_max, self.m, self.p_total, self.q
        )?;
        writeln!(f)?;
        write!(f, "|")?;
        for t in 0..self.d_max {
            write!(f, "{:03}|", t)?;
        }
        for job in self.jobs.iter() {
            write!(f, "\n{}", job)?;
        }
        Ok(())
    }
}
