# Node Storage Availability Optimization

## Background

This repository contains a Rust-based simulation of two and three-tiered data availability
optimization of online algorithms using randomized competitive analysis proposed by Anna Karlin
in [Competitive Randomized Algorithms for Non-Uniform Problems](http://courses.csail.mit.edu/6.895/fall03/handouts/papers/karlin.pdf).

This project is meant to derive empirical results that can be cross-checked against the calculus-heavy 
math used in the theoretical approach.

## Installing

- Rust cargo 1.65.0 (4bc8f24d3 2022-10-20)

Run with `cargo run` and test with `cargo test`

## Two-Tiered Competitive Analysis

The idea is that data availability nodes can either keep data or discard data. Keeping data has a
unit time cost, but recovering from a discarded state has a bigger recovery cost. The node can
use a storage policy that optimizes cost. However, if an adversary knows the policy a node uses,
it can always defeat it. This is equivalent to the spin-block problem posed by Karlin in her paper above,
as it is an online algorithm: the node does not know the access policy in advance, but a strong adversary does.

If the node could know the access policy in advance (an offline algorithm), it would be able to have 
an optimal policy, however, this is not possible. Karlin shows there is a theoretical bound that
an online algorithm cannot do better than 2x of the offline one given a strong adversary.

This repository reproduces Karlin's analysis and also implements a randomized policy approach, where nodes
discard if the time since last access is some $D$ that is drawn from an optimal probability distribution
for the problem. We implement the Karlin distribution and use it to build a randomized policy that performs
better than the 2x bound as shown in the paper.

![Image](https://i.imgur.com/9jL2JW6.png)

We use a monte-carlo based sampling method to draw integers from the Karlin distribution, equivalent
to box sampling to get integers within a range. A Jupyter notebook is included under `notebooks` that
shows the histogram and pdf properties.

```
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
```

![Image](https://i.imgur.com/8kxKNgk.png)

## Three-Tiered Problems

We also extend the analysis to a three-tiered approach, where the node has three modes: keep, discard, or compress.
The idea is that compressing incurs a cheaper cost than keeping, and recovering from a compressed mode is cheaper
than recovering from a discarded mode. Much of Karlin's analysis still applies and we compute competitive ratios
for both a randomized and deterministic policy for the three-tiered problem.

## Running

`cargo run` checks the two-tiered deterministic vs. randomized approaches to show how the randomized approach
outperforms across a variety of access lists.

```
ratio: deterministic=1.80, randomized=1.60
```














