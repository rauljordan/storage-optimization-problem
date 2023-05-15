use crate::{Algorithm, Policy};
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
    fn total_accrued_cost(&self) -> u64 {
        self.accrued_cost
    }
}
