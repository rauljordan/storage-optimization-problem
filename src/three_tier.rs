use crate::{karlin, Algorithm, Policy};
use std::iter::Peekable;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn three_tier_instance() {
        let costs = Costs {
            keep_time_cost: 1.0,
            compressed_time_cost: 0.5,
            recover_from_compressed_cost: 2.0,
            recover_from_discard_cost: 3.0,
        };
        let access_list = vec![4, 8, 12];
        let online = KarlinInstance::new(costs.clone());
        let num_ticks = 12;
        let ratio = calculate_competitive_ratio(online, costs, access_list, num_ticks);
        eprintln!("{}", ratio);
        assert_eq!(1, 2);
    }
}

#[derive(Debug, Clone)]
pub struct Costs {
    pub keep_time_cost: f64,
    pub compressed_time_cost: f64,
    pub recover_from_compressed_cost: f64,
    pub recover_from_discard_cost: f64,
}

#[derive(Debug, Clone)]
pub struct KarlinInstance {
    t: u64,
    accrued_cost: f64,
    costs: Costs,
    policy: Policy,
    last_access: u64,
    t_to_wait_before_discard: u64,
    t_to_wait_before_compress: u64,
}

impl KarlinInstance {
    pub fn new(costs: Costs) -> KarlinInstance {
        // The cost to keep compressed data is less than the normal keep cost.
        assert!(costs.compressed_time_cost < 1.0);
        // Recovering from a discard is more expensive than from a compressed state.
        assert!(costs.recover_from_compressed_cost < costs.recover_from_discard_cost);
        let cc = costs.recover_from_compressed_cost;
        let dc = costs.recover_from_discard_cost;
        Self {
            t: 0,
            costs,
            accrued_cost: 0.0,
            policy: Policy::Keep,
            last_access: 0,
            t_to_wait_before_discard: karlin::sample(dc as u64),
            t_to_wait_before_compress: karlin::sample(cc as u64),
        }
    }
}

impl Algorithm for KarlinInstance {
    fn tick(&mut self, access: bool) {
        self.t += 1;
        // Check if we need to change our policy. Should only do this if
        // we are in keep mode for the instance.
        if matches!(self.policy, Policy::Keep) {
            let time_elapsed = self.t - self.last_access;
            let should_discard = time_elapsed >= self.t_to_wait_before_discard;
            let should_compress = time_elapsed >= self.t_to_wait_before_compress;
            if should_discard {
                self.policy = Policy::Discard;
            } else if should_compress {
                self.policy = Policy::Compress;
            }
        }
        // if no access, charge normal time costs if applicable.
        if !access {
            match self.policy {
                Policy::Keep => self.accrued_cost += self.costs.keep_time_cost,
                Policy::Compress => self.accrued_cost += self.costs.compressed_time_cost,
                Policy::Discard => {}
            }
            return;
        }
        self.last_access = self.t;
        self.t_to_wait_before_discard = karlin::sample(self.costs.recover_from_discard_cost as u64);
        self.t_to_wait_before_compress =
            karlin::sample(self.costs.recover_from_compressed_cost as u64);

        // Incur a recovery cost if necessary.
        match self.policy {
            Policy::Compress => self.accrued_cost += self.costs.recover_from_compressed_cost,
            Policy::Discard => self.accrued_cost += self.costs.recover_from_discard_cost,
            Policy::Keep => {}
        }
        self.policy = Policy::Keep;
    }
    fn total_accrued_cost(&self) -> u64 {
        self.accrued_cost as u64
    }
}

#[derive(Debug, Clone)]
pub struct OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    t: u64,
    access_list: Peekable<T>,
    accrued_cost: f64,
    costs: Costs,
    policy: Policy,
}

impl<T> OfflineInstance<T>
where
    T: Iterator<Item = u64>,
{
    pub fn new(costs: Costs, access_list: Peekable<T>) -> OfflineInstance<T> {
        // The cost to keep compressed data is less than the normal keep cost.
        assert!(costs.compressed_time_cost < 1.0);
        // Recovering from a discard is more expensive than from a compressed state.
        assert!(costs.recover_from_compressed_cost < costs.recover_from_discard_cost);
        Self {
            t: 0,
            access_list,
            costs,
            accrued_cost: 0.0,
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
        let Some(next_access) = self.access_list.peek() else {
            return;
        };
        // Check if we need to change our policy. Should only do this if
        // we are in keep mode for the instance.
        if matches!(self.policy, Policy::Keep) {
            let next_access = *next_access as f64;
            let keep_threshold =
                self.costs.recover_from_compressed_cost / (1.0 - self.costs.compressed_time_cost);
            if next_access <= keep_threshold {
                self.policy = Policy::Keep;
            }
            let compress_threshold = (self.costs.recover_from_discard_cost
                - self.costs.recover_from_compressed_cost)
                / self.costs.compressed_time_cost;
            if keep_threshold <= next_access && next_access <= compress_threshold {
                self.policy = Policy::Compress;
            }
            if next_access > compress_threshold {
                self.policy = Policy::Discard;
            }
        }
        // if no access, charge normal time costs if applicable.
        if !access {
            match self.policy {
                Policy::Keep => self.accrued_cost += self.costs.keep_time_cost,
                Policy::Compress => self.accrued_cost += self.costs.compressed_time_cost,
                Policy::Discard => {}
            }
            return;
        }

        // Advance the access list iterator.
        let _ = self.access_list.next();

        // Incur a recovery cost if necessary.
        match self.policy {
            Policy::Compress => self.accrued_cost += self.costs.recover_from_compressed_cost,
            Policy::Discard => self.accrued_cost += self.costs.recover_from_discard_cost,
            Policy::Keep => {}
        }
        self.policy = Policy::Keep;
    }
    fn total_accrued_cost(&self) -> u64 {
        self.accrued_cost as u64
    }
}

pub fn calculate_competitive_ratio<T: Algorithm>(
    instance: T,
    costs: Costs,
    access_list: Vec<u64>,
    num_ticks: u64,
) -> f64 {
    // Offline, omniscient instance.
    let offline = OfflineInstance::new(costs, access_list.clone().into_iter().peekable());
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
