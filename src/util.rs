use rand::{thread_rng, Rng};
use std::iter::repeat;

pub fn generate_access_list(len: usize, max_value: u64) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut access_list: Vec<u64> = repeat(0)
        .take(len)
        .map(|_: u64| rng.gen_range(1..=max_value as u64))
        .collect();
    access_list.sort();
    access_list.dedup();
    access_list
}
