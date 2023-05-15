use crate::{karlin, Algorithm, Policy};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct OfflineInstance<T>
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
    pub fn new(keep_cost: u64, recover_cost: u64, access_list: Peekable<T>) -> OfflineInstance<T> {
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
        if !access {
            if matches!(self.policy, Policy::Keep) {
                self.accrued_cost += self.keep_cost;
            }
            return;
        }
        let _ = self.access_list.next();
        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
        }
    }
    fn total_accrued_cost(&self) -> u64 {
        self.accrued_cost
    }
}

#[derive(Debug, Clone)]
pub struct NaiveInstance {
    t: u64,
    keep_cost: u64,
    recover_cost: u64,
    policy: Policy,
    accrued_cost: u64,
    last_access: u64,
}

impl NaiveInstance {
    pub fn new(keep_cost: u64, recover_cost: u64) -> Self {
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
        if !access {
            if matches!(self.policy, Policy::Keep) {
                self.accrued_cost += self.keep_cost;
            }
            return;
        }
        self.last_access = self.t;

        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
        }
    }
    fn total_accrued_cost(&self) -> u64 {
        self.accrued_cost
    }
}

#[derive(Debug, Clone)]
pub struct KarlinInstance {
    t: u64,
    keep_cost: u64,
    recover_cost: u64,
    policy: Policy,
    accrued_cost: u64,
    last_access: u64,
    t_to_wait_before_discard: u64,
}

impl KarlinInstance {
    pub fn new(keep_cost: u64, recover_cost: u64) -> Self {
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
        // Randomized competitive algorithm. If time since last access
        // is >= D, we discard where 0 <= D <= C. We sample this D from a Karlin distribution
        // after each access occurs.
        let time_elapsed = self.t - self.last_access;
        let should_discard = time_elapsed >= self.t_to_wait_before_discard;
        if matches!(self.policy, Policy::Keep) && should_discard {
            self.policy = Policy::Discard;
        }
        if !access {
            if matches!(self.policy, Policy::Keep) {
                self.accrued_cost += self.keep_cost;
            }
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
    fn total_accrued_cost(&self) -> u64 {
        self.accrued_cost
    }
}

pub fn calculate_competitive_ratio<T: Algorithm>(
    instance: T,
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
    let mut sim = crate::Simulator::new(access_list.clone(), offline);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let offline_cost = sim.node.total_accrued_cost();

    // Online instance.
    let mut sim = crate::Simulator::new(access_list, instance);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let online_cost = sim.node.total_accrued_cost();

    // Competitive ratio.
    online_cost as f64 / offline_cost as f64
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn two_competitive() {
        let keep_cost = 1u64;
        let recover_cost = 3u64;
        let num_ticks = 11;
        let access_list = vec![4, 8, 12];
        let online_instance = NaiveInstance::new(keep_cost, recover_cost);
        let competitive_ratio = calculate_competitive_ratio(
            online_instance,
            keep_cost,
            recover_cost,
            access_list,
            num_ticks,
        );
        assert_eq!(2.0, competitive_ratio);
    }
    #[test]
    fn randomized_competitive() {
        let keep_cost = 1u64;
        let recover_cost = 3u64;
        let num_ticks = 11;
        let access_list = vec![4, 8, 12];
        let online_instance = KarlinInstance::new(keep_cost, recover_cost);
        let competitive_ratio = calculate_competitive_ratio(
            online_instance,
            keep_cost,
            recover_cost,
            access_list,
            num_ticks,
        );
        assert!(competitive_ratio < 1.67);
    }
}
