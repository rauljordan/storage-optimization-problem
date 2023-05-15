mod karlin;
mod three_tier;
mod two_tier;
mod util;

use two_tier::{calculate_competitive_ratio, KarlinInstance, NaiveInstance};

#[derive(Debug, Clone)]
pub enum Policy {
    Keep,
    Discard,
    Compress,
}

pub trait Algorithm {
    fn tick(&mut self, access: bool);
    fn total_accrued_cost(&self) -> u64;
}

#[derive(Debug)]
pub struct Simulator<T: Algorithm> {
    t: u64,
    access: Vec<u64>,
    node: T,
}

impl<T: Algorithm> Simulator<T> {
    pub fn new(access: Vec<u64>, node: T) -> Self {
        Self { t: 0, access, node }
    }
    pub fn tick(&mut self) {
        self.t += 1;
        let should_access = self.access.contains(&self.t);
        self.node.tick(should_access);
    }
}

/// We show the randomized strategy for the two-tier problem across
/// a variety of random access lists.
fn main() {
    let keep_cost = 1u64;
    let recover_cost = 3u64;
    for _ in 0..100 {
        let access_list = util::generate_access_list(10, 100);
        let num_ticks = access_list.last().unwrap().clone();
        let online = NaiveInstance::new(keep_cost, recover_cost);
        let deterministic_competitive_ratio = calculate_competitive_ratio(
            online,
            keep_cost,
            recover_cost,
            access_list.clone(),
            num_ticks,
        );
        let online = KarlinInstance::new(keep_cost, recover_cost);
        let randomized_competitive_ratio = calculate_competitive_ratio(
            online,
            keep_cost,
            recover_cost,
            access_list.clone(),
            num_ticks,
        );
        println!(
            "ratio: deterministic={:.2}, randomized={:.2}",
            deterministic_competitive_ratio, randomized_competitive_ratio,
        );
    }
}
