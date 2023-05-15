use rand::{thread_rng, Rng};

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn karlin_expected_value() {
        let cost = 1;
        let res = pdf(cost, cost);
        assert_eq!(format!("{:.2}", res), "1.58");
    }
}

/// Parametrized by c, creates a pdf with an expected value of (1 / ((e - 1) * C)).
pub fn pdf(t: u64, c: u64) -> f64 {
    let e = std::f64::consts::E;
    let lhs = 1.0 / ((e - 1.0) * c as f64);
    let rhs = e.powf(t as f64 / c as f64);
    lhs * rhs
}

/// Monte carlo sampling method for the karlin pdf.
pub fn sample(cost: u64) -> u64 {
    let max_iters = 10_000;
    let mut rng = thread_rng();
    let max_value: f64 = pdf(cost, cost);
    for _ in 0..max_iters {
        let rand_x = rng.gen_range(0..=cost);
        let rand_y = max_value * rng.gen_range(0.0f64..1.0f64);
        let calc_y = pdf(rand_x, cost);
        if rand_y <= calc_y {
            return rand_x;
        }
    }
    panic!("could not find through {} iterations", max_iters)
}
