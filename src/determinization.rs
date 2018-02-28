use std::collections::HashMap;
use std::hash::Hash;
use std::ops::AddAssign;
use rand::Rng;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;

// For a game of Hanabi, determinization is straightforward: assign all the
// unseen cards identities that are consistent with the seen cards.
// For inference, some of the cards will have non-uniform prior distributions,
// but this will usually be a small number of cards.

pub fn determinize<A, B, D, R>(
    unknowns: &[A],
    remaining_counts: &HashMap<B, usize>,
    priors: &HashMap<A, D>,
    rng: &mut R,
) -> HashMap<A, B>
    where
    A: Hash + Eq + Copy,
    B: Hash + Eq + Copy,
    D: IndependentSample<B>,
    R: Rng,
{
    // For our purposes, there will usually only be a few cards with priors
    // (It is limited to the cards that are in the hand of the player doing
    // the determinization).
    // We do rejection sampling on those cards, and then shuffle the remainder.
    // As a result, priors should not include uniform distributions, as that will
    // cause significantly more rejection sampling steps.
    let mut assignment = HashMap::new();
    let mut counts: HashMap<B, usize> = HashMap::new();
    // Sample each prior until no output is used too many times.
    loop {
        assignment.clear();
        counts.clear();
        for (&a, d) in priors.iter() {
            let b = d.ind_sample(rng);
            assignment.insert(a, b);
            counts.entry(b).or_insert(0).add_assign(1);
        }
        let mut reroll = false;
        for (b, n) in counts.iter() {
            if n > remaining_counts.get(b).unwrap_or(&0) {
                reroll = true;
                break;
            }
        }
        if !reroll {
            break;
        }
    }

    // Build a list of all the remaining available outputs.
    let mut available_list: Vec<B> = Vec::new();

    for (&b, &n) in remaining_counts.iter() {
        let used_count = counts.get(&b).unwrap_or(&0);
        for _ in 0..(n - used_count) {
            available_list.push(b);
        }
    }

    for &a in unknowns.iter() {
        if assignment.contains_key(&a) {
            continue;
        }

        if available_list.is_empty() {
            panic!("Not enough values remaining for determinization");
        }

        let i = Range::new(0, available_list.len()).ind_sample(rng);
        let b = available_list.swap_remove(i);
        assignment.insert(a, b);
    }

    assignment
}
