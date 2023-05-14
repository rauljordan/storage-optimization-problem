use ordered_float::NotNan;
use rand::{thread_rng, Rng};
use std::iter::repeat;
use std::iter::Peekable;

mod karlin;

/// We show the 2-competitive strategy for a deterministic
/// approach will never have a competitive ratio worse than 2x
/// the offline algorithm for a variety of access list sizes and random
/// access lists.
fn main() {
    let keep_cost = 1u64;
    let recover_cost = 10u64;
    let max_access_list_size = 50;
    let num_iters = 1000;
    let mut max_ratios = vec![];
    for i in 1..max_access_list_size {
        let mut ratios = vec![];
        for _ in 0..num_iters {
            let access_list = generate_access_list(i, max_access_list_size as u64);
            let num_ticks = access_list.last().unwrap().clone();
            let competitive_ratio =
                compare_randomized_approach(keep_cost, recover_cost, access_list, num_ticks);
            ratios.push(competitive_ratio);
        }
        let max_val = ratios
            .into_iter()
            .map(NotNan::new)
            .filter_map(Result::ok)
            .max()
            .unwrap();
        println!(
            "access_list_size={}, iters={}, max_competitive_ratio={}",
            i, num_iters, *max_val,
        );
        max_ratios.push(max_val);
    }
    let max_ratio = max_ratios.into_iter().max().unwrap();
    println!(
        "max_competitive_ratio={} across all attempts and access list sizes",
        *max_ratio,
    );
}

fn compare_deterministic_approach(
    keep_cost: u64,
    recover_cost: u64,
    access_list: Vec<u64>,
    num_ticks: u64,
) -> f64 {
    // Offline, omniscient instance.
    let offline = OfflineInstance::new(
        keep_cost,
        recover_cost,
        access_list.clone().into_iter().peekable(),
    );
    let mut sim = Simulator::new(access_list.clone(), offline);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let offline_cost = sim.node.accrued_cost;

    // Standard, 2-competitive online instance.
    let online = NaiveInstance::new(keep_cost, recover_cost);
    let mut sim = Simulator::new(access_list, online);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let online_cost = sim.node.accrued_cost;

    // Competitive ratio.
    online_cost as f64 / offline_cost as f64
}

fn compare_randomized_approach(
    keep_cost: u64,
    recover_cost: u64,
    access_list: Vec<u64>,
    num_ticks: u64,
) -> f64 {
    // Offline, omniscient instance.
    let offline = OfflineInstance::new(
        keep_cost,
        recover_cost,
        access_list.clone().into_iter().peekable(),
    );
    let mut sim = Simulator::new(access_list.clone(), offline);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let offline_cost = sim.node.accrued_cost;

    // Randomized, karlin online approach.
    let online = KarlinInstance::new(keep_cost, recover_cost);
    let mut sim = Simulator::new(access_list, online);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let online_cost = sim.node.accrued_cost;

    // Competitive ratio.
    online_cost as f64 / offline_cost as f64
}

#[derive(Debug, Clone)]
enum Policy {
    Keep,
    Discard,
}

trait Algorithm {
    fn tick(&mut self, access: bool);
}

#[derive(Debug)]
struct Simulator<T: Algorithm> {
    t: u64,
    access: Vec<u64>,
    node: T,
}

impl<T: Algorithm> Simulator<T> {
    fn new(access: Vec<u64>, node: T) -> Self {
        Self { t: 0, access, node }
    }
    fn tick(&mut self) {
        self.t += 1;
        let should_access = self.access.contains(&self.t);
        self.node.tick(should_access);
    }
}

#[derive(Debug, Clone)]
struct OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    t: u64,
    access_list: Peekable<T>,
    keep_cost: u64,
    recover_cost: u64,
    accrued_cost: u64,
    policy: Policy,
}

impl<T> OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    fn new(keep_cost: u64, recover_cost: u64, access_list: Peekable<T>) -> OfflineInstance<T> {
        Self {
            t: 0,
            access_list,
            keep_cost,
            recover_cost,
            accrued_cost: 0,
            policy: Policy::Keep,
        }
    }
}

impl<T> Algorithm for OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    fn tick(&mut self, access: bool) {
        self.t += 1;
        // Omniscient algorithm: if we are keeping, and if the time to
        // next access is > C, then discard
        match (&self.policy, self.access_list.peek()) {
            (Policy::Keep, Some(&elem)) => {
                let time_to_next_access = elem - self.t;
                if time_to_next_access >= self.recover_cost {
                    self.policy = Policy::Discard;
                }
            }
            _ => {}
        }
        if matches!(self.policy, Policy::Keep) {
            self.accrued_cost += self.keep_cost;
        }
        if !access {
            return;
        }
        let _ = self.access_list.next();
        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
        }
    }
}

#[derive(Debug, Clone)]
struct NaiveInstance {
    t: u64,
    keep_cost: u64,
    recover_cost: u64,
    policy: Policy,
    accrued_cost: u64,
    last_access: u64,
}

impl NaiveInstance {
    fn new(keep_cost: u64, recover_cost: u64) -> Self {
        Self {
            t: 0,
            last_access: 0,
            keep_cost,
            recover_cost,
            policy: Policy::Keep,
            accrued_cost: 0,
        }
    }
}

impl Algorithm for NaiveInstance {
    fn tick(&mut self, access: bool) {
        self.t += 1;
        // 2-competitive algorithm. If time since last access
        // is >= recover cost, then we should discard.
        let should_discard = (self.t - self.last_access) >= self.recover_cost;
        if matches!(self.policy, Policy::Keep) && should_discard {
            self.policy = Policy::Discard;
        }
        if matches!(self.policy, Policy::Keep) {
            self.accrued_cost += self.keep_cost;
        }
        if !access {
            return;
        }
        self.last_access = self.t;

        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
        }
    }
}

#[derive(Debug, Clone)]
struct KarlinInstance {
    t: u64,
    keep_cost: u64,
    recover_cost: u64,
    policy: Policy,
    accrued_cost: u64,
    last_access: u64,
    t_to_wait_before_discard: u64,
}

impl KarlinInstance {
    fn new(keep_cost: u64, recover_cost: u64) -> Self {
        Self {
            t: 0,
            last_access: 0,
            keep_cost,
            recover_cost,
            policy: Policy::Keep,
            accrued_cost: 0,
            t_to_wait_before_discard: karlin::sample(recover_cost),
        }
    }
}

impl Algorithm for KarlinInstance {
    fn tick(&mut self, access: bool) {
        self.t += 1;
        // 2-competitive algorithm. If time since last access
        // is >= D where 0 <= D <= C. We sample this D from a Karlin distribution
        // after each access occurs.
        let time_elapsed = self.t - self.last_access;
        let should_discard = time_elapsed >= self.t_to_wait_before_discard;
        if matches!(self.policy, Policy::Keep) && should_discard {
            self.policy = Policy::Discard;
        }
        if matches!(self.policy, Policy::Keep) {
            self.accrued_cost += self.keep_cost;
        }
        if !access {
            return;
        }
        self.t_to_wait_before_discard = karlin::sample(self.recover_cost);
        self.last_access = self.t;

        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
        }
    }
}

fn generate_access_list(len: usize, max: u64) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut access_list: Vec<u64> = repeat(0)
        .take(len)
        .map(|_: u64| rng.gen_range(1..=max as u64))
        .collect();
    access_list.sort();
    access_list.dedup();
    access_list
}
