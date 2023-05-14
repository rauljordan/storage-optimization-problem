use rand::{thread_rng, Rng};
use std::iter::repeat;
use std::iter::Peekable;

fn main() {
    let cost_per_keep = 1u64;
    let cost_per_recover = 1u64;

    let num_accesses = 20;
    let num_ticks = 40;
    let access_list: Vec<u64> = generate_access_list(num_accesses, num_ticks);
    dbg!(access_list.clone());

    // Offline instance.
    let offline = OfflineInstance::new(access_list.clone().into_iter().peekable());
    let mut sim = Simulator::new(access_list.clone(), offline);
    for _ in 0..num_ticks {
        sim.tick();
    }
    dbg!(&sim.node);
    let offline_cost = sim.node.accrued_cost;

    // Attempt a naive, online instance.
    let online = NaiveInstance::new(cost_per_keep, cost_per_recover);
    let mut sim = Simulator::new(access_list, online);
    for _ in 0..num_ticks {
        sim.tick();
    }
    dbg!(&sim.node);
    let online_cost = sim.node.accrued_cost;
    // Attempt a randomized, online instance.
    println!(
        "Competitive ratio = {}",
        online_cost as f64 / offline_cost as f64
    );
}

#[derive(Debug, Clone)]
enum Policy {
    Keep,
    Discard,
}

trait Algorithm {
    fn tick(&mut self);
    fn access(&mut self);
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
        if self.access.contains(&self.t) {
            self.node.access();
        }
        self.node.tick();
    }
}

#[derive(Debug, Clone)]
struct OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    t: u64,
    access_list: Peekable<T>,
    cost_per_tick: u64,
    recover_cost: u64,
    accrued_cost: u64,
    policy: Policy,
}

impl<T> OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    fn new(access_list: Peekable<T>) -> OfflineInstance<T> {
        Self {
            t: 0,
            access_list,
            cost_per_tick: 1,
            recover_cost: 1,
            accrued_cost: 0,
            policy: Policy::Keep,
        }
    }
}

impl<T> Algorithm for OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    fn tick(&mut self) {
        self.t += 1;
        if matches!(self.policy, Policy::Keep) {
            self.accrued_cost += self.cost_per_tick;
        }
    }
    fn access(&mut self) {
        self.accrued_cost += self.recover_cost;
        let _ = self.access_list.next();
        match self.access_list.peek() {
            Some(&elem) => {
                let time_to_next_access = elem - self.t;
                if time_to_next_access > self.cost_per_tick {
                    self.policy = Policy::Discard;
                } else {
                    self.policy = Policy::Keep;
                }
            }
            None => {}
        }
    }
}

#[derive(Debug, Clone)]
struct NaiveInstance {
    t: u64,
    cost_per_tick: u64,
    recover_cost: u64,
    policy: Policy,
    accrued_cost: u64,
    last_access: u64,
}

impl NaiveInstance {
    fn new(cost_per_tick: u64, recover_cost: u64) -> Self {
        Self {
            t: 0,
            last_access: 0,
            cost_per_tick,
            recover_cost,
            policy: Policy::Keep,
            accrued_cost: 0,
        }
    }
}

impl Algorithm for NaiveInstance {
    fn tick(&mut self) {
        self.t += 1;
        // Accrue costs based on policy.
        match self.policy {
            Policy::Keep => self.accrued_cost += self.cost_per_tick,
            Policy::Discard => {}
        }
    }
    fn access(&mut self) {
        self.last_access = self.t;
        self.accrued_cost += self.recover_cost;
        if self.last_access > self.cost_per_tick {
            self.policy = Policy::Discard;
        } else {
            self.policy = Policy::Keep;
        }
    }
}

fn generate_access_list(len: usize, num_ticks: u64) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut access_list: Vec<u64> = repeat(0)
        .take(len)
        .map(|_: u64| rng.gen_range(0..num_ticks as u64))
        .collect();
    access_list.sort();
    access_list.dedup();
    access_list
}
