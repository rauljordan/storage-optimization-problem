use std::iter::Peekable;

#[cfg(test)]
mod test {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::iter::repeat;
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
    fn karlin_pdf(t: u64, c: u64) -> f64 {
        let e = std::f64::consts::E;
        let lhs = 1.0 / ((e - 1.0) * c as f64);
        let rhs = e.powf(t as f64 / c as f64);
        lhs * rhs
    }

    // TODO: Fix up box method.
    fn sample_karlin(max_iters: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let max_value: f64 = 1.58;
        for _ in 0..max_iters {
            let rand_x = rng.gen_range(0..40);
            let rand_y = max_value * rng.gen_range(0.0f64..=1.0f64);
            let calc_y = karlin_pdf(rand_x, 1);
            println!("randx={}, randy={}, calcy={}", rand_x, rand_y, calc_y);
            if rand_y <= calc_y {
                return rand_x;
            }
        }
        panic!("could not find in iterations")
    }
    #[test]
    fn check_karlin_ev() {
        let res = karlin_pdf(1, 1);
        assert_eq!(format!("{:.2}", res), "1.58");
    }
}

fn main() {
    let keep_cost = 1u64;
    let recover_cost = 5u64;
    let num_ticks = 25;
    let access_list = vec![7, 13, 19, 25];
    let (online_cost, offline_cost, ratio) =
        compare(keep_cost, recover_cost, access_list, num_ticks);
    println!(
        "online_cost={}, offline_cost={}, competitive_ratio={:.2}",
        online_cost, offline_cost, ratio
    );
}

fn compare(
    keep_cost: u64,
    recover_cost: u64,
    access_list: Vec<u64>,
    num_ticks: u64,
) -> (u64, u64, f64) {
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

    // Naive, 2-competitive online instance.
    let online = NaiveInstance::new(keep_cost, recover_cost);
    let mut sim = Simulator::new(access_list, online);
    for _ in 0..num_ticks {
        sim.tick();
    }
    let online_cost = sim.node.accrued_cost;
    (
        online_cost,
        offline_cost,
        online_cost as f64 / offline_cost as f64,
    )
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
                    println!(
                        "Offline: Discarding time={}, total={}",
                        self.t, self.accrued_cost
                    );
                    self.policy = Policy::Discard;
                }
            }
            _ => {}
        }
        if matches!(self.policy, Policy::Keep) {
            self.accrued_cost += self.keep_cost;
            println!(
                "Offline: Keeping time={}, total={}",
                self.t, self.accrued_cost
            );
        }
        if !access {
            return;
        }
        let _ = self.access_list.next();
        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
            println!(
                "Offline: Recovering time={}, total={}",
                self.t, self.accrued_cost
            );
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
            println!(
                "Online: Discarding time={}, total={}",
                self.t, self.accrued_cost
            );
            self.policy = Policy::Discard;
        }
        if matches!(self.policy, Policy::Keep) {
            self.accrued_cost += self.keep_cost;
            println!(
                "Online: Keeping time={}, total={}",
                self.t, self.accrued_cost
            );
        }
        if !access {
            return;
        }
        self.last_access = self.t;

        // Incur a recovery cost if necessary.
        if matches!(self.policy, Policy::Discard) {
            self.accrued_cost += self.recover_cost;
            self.policy = Policy::Keep;
            println!(
                "Online: Recovering time={}, total={}",
                self.t, self.accrued_cost
            );
        }
    }
}
