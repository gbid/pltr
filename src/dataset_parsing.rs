use crate::types::{Job, Instance};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct CsvJob {
    M: usize,
    N: usize,
    r: String,
    d: String,
    p: String,
}


fn parse_vector_string(s: String) -> Vec<usize> {
    s.trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|s| s.trim().parse::<usize>().unwrap())
        .collect()
}

fn parse_matrix_string(s: String, n: usize) -> Vec<usize> {
    let rows: Vec<Vec<usize>> = s.split(';')
        .map(|row_str| parse_vector_string(row_str.to_string()))
        .collect();

    let mut min_times: Vec<usize> = vec![usize::MAX; n];

    for row in rows {
        for (j, time) in row.iter().enumerate() {
            if *time < min_times[j] {
                min_times[j] = *time;
            }
        }
    }

    min_times
}

pub fn parse_csv_to_instance(file_path: &str) -> Result<Vec<Instance>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(file_path)?;
    let mut instances = Vec::new();

    for result in reader.deserialize() {
        let record: CsvJob = result?;
        let r = parse_vector_string(record.r);
        let d = parse_vector_string(record.d);
        let p = parse_matrix_string(record.p, record.N);
        
        let jobs = r.iter().zip(&d).zip(&p).map(|((r, d), p)| {
            Job::new(*r, *d, *p)
        }).collect();
        instances.push(Instance::new(jobs, record.M, 1));
    }
    Ok(instances)
}
