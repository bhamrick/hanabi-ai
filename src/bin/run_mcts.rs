extern crate hanabi_ai;
extern crate rand;
extern crate rayon;

use hanabi_ai::hanabi::{deck_distribution, Action, ActionResult, Card, GameState};
use hanabi_ai::basic_mcts::MctsState;

use rand::distributions::{IndependentSample, Range};

use rayon::prelude::*;

fn main() {
    let batch_size: usize = 20;
    let batches_per_move: usize = 10000;
    let mut rng = rand::thread_rng();

    let mut deck: Vec<Card> = Vec::new();
    for (&c, &count) in deck_distribution().iter() {
        for _ in 0..count {
            deck.push(c);
        }
    }

    // Shuffle the deck
    for i in 0..deck.len() {
        let j = Range::new(0, i+1).ind_sample(&mut rng);
        deck.swap(i, j);
    }

    println!("Deck order:");
    for c in deck.iter() {
        println!("{:?}", c);
    }

    println!("");
    let mut current_state = GameState::initial(&deck);
    let mut batch_updates: Vec<(Vec<(u64, Action)>, f64)> = Vec::new();
    let result = loop {
        let current_player = current_state.current_player();
        let current_view = current_state.current_view();

        let mut mcts = MctsState::new(current_view, 1.4);
        for _ in 0..batches_per_move {
            (0..batch_size).into_par_iter()
                .map(|_| {
                    let mut rng = rand::thread_rng();
                    mcts.run_playout(&mut rng)
                })
                .collect_into_vec(&mut batch_updates);
            for (updates, result) in batch_updates.drain(..) {
                mcts.update(updates, result);
            }
        }

        let action = mcts.choose_action(&mut rng);

        println!("{:?}: {:?}", current_player, action);

        match current_state.act(action) {
            ActionResult::Acted(completed_action) => {
                println!("{:?}", completed_action);
            },
            ActionResult::Illegal(reason) => {
                println!("{:?}", current_state);
                panic!("Chose an illegal move: {:?}", reason);
            },
            ActionResult::Error(e) => {
                panic!("Encountered an error: {:?}", e);
            },
            ActionResult::Finished(score) => {
                break score;
            },
        }
    };
    println!("Final result: {}", result);
}
