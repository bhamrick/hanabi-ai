use std::collections::HashMap;
use rand;

use hanabi::{Card, GameState, Suit, Rank, Player, Clue, Action, ActionResult};
use basic_mcts::MctsState;

const SAMPLE_DECK : [Card; 50] = [
    Card { suit: Suit::Yellow, rank: Rank::One },
    Card { suit: Suit::Yellow, rank: Rank::Four },
    Card { suit: Suit::Red, rank: Rank::Three },
    Card { suit: Suit::Green, rank: Rank::One },
    Card { suit: Suit::Red, rank: Rank::One },
    Card { suit: Suit::Purple, rank: Rank::One },
    Card { suit: Suit::Green, rank: Rank::Five },
    Card { suit: Suit::Blue, rank: Rank::Five },
    Card { suit: Suit::Green, rank: Rank::Four },
    Card { suit: Suit::Blue, rank: Rank::Four },
    Card { suit: Suit::Blue, rank: Rank::One },
    Card { suit: Suit::Red, rank: Rank::Three },
    Card { suit: Suit::Red, rank: Rank::Four },
    Card { suit: Suit::Red, rank: Rank::One },
    Card { suit: Suit::Blue, rank: Rank::Four },
    Card { suit: Suit::Yellow, rank: Rank::Three },
    Card { suit: Suit::Green, rank: Rank::Three },
    Card { suit: Suit::Red, rank: Rank::One },
    Card { suit: Suit::Blue, rank: Rank::One },
    Card { suit: Suit::Green, rank: Rank::One },
    Card { suit: Suit::Purple, rank: Rank::Two },
    Card { suit: Suit::Red, rank: Rank::Two },
    Card { suit: Suit::Red, rank: Rank::Five },
    Card { suit: Suit::Blue, rank: Rank::Three },
    Card { suit: Suit::Yellow, rank: Rank::Four },
    Card { suit: Suit::Purple, rank: Rank::One },
    Card { suit: Suit::Yellow, rank: Rank::One },
    Card { suit: Suit::Yellow, rank: Rank::One },
    Card { suit: Suit::Green, rank: Rank::Four },
    Card { suit: Suit::Green, rank: Rank::One },
    Card { suit: Suit::Yellow, rank: Rank::Three },
    Card { suit: Suit::Blue, rank: Rank::Three },
    Card { suit: Suit::Purple, rank: Rank::Four },
    Card { suit: Suit::Green, rank: Rank::Three },
    Card { suit: Suit::Purple, rank: Rank::Three },
    Card { suit: Suit::Yellow, rank: Rank::Two },
    Card { suit: Suit::Red, rank: Rank::Two },
    Card { suit: Suit::Purple, rank: Rank::Five },
    Card { suit: Suit::Blue, rank: Rank::Two },
    Card { suit: Suit::Blue, rank: Rank::One },
    Card { suit: Suit::Green, rank: Rank::Two },
    Card { suit: Suit::Yellow, rank: Rank::Five },
    Card { suit: Suit::Purple, rank: Rank::One },
    Card { suit: Suit::Yellow, rank: Rank::Two },
    Card { suit: Suit::Blue, rank: Rank::Two },
    Card { suit: Suit::Purple, rank: Rank::Two },
    Card { suit: Suit::Purple, rank: Rank::Four },
    Card { suit: Suit::Purple, rank: Rank::Three },
    Card { suit: Suit::Green, rank: Rank::Two },
    Card { suit: Suit::Red, rank: Rank::Four },
];

/*
#[test]
fn basic_test() {
    let state = GameState::initial(&SAMPLE_DECK);

    println!("{:?}", state);
    println!("");
    let alice_view = state.player_view(Player::Alice);
    println!("{:?}", alice_view);

    let state2 = match state.act(Action::Clue(Player::Bob, Clue::Rank(Rank::One))) {
        ActionResult::Acted(complete_action, new_state) => {
            println!("{:?}", complete_action);
            println!("");
            println!("{:?}", new_state);
            println!("");
            println!("{:?}", new_state.player_view(Player::Bob));

            new_state
        },
        _ => panic!("Acting failed!"),
    };

    let state3 = match state2.act(Action::Play(1)) {
        ActionResult::Acted(complete_action, new_state) => {
            println!("{:?}", complete_action);
            println!("");
            println!("{:?}", new_state);
            println!("");
            println!("{:?}", new_state.player_view(Player::Cathy));

            new_state
        },
        _ => panic!("Acting failed!"),
    };

    let final_view = state3.current_view();
    let mut rng = rand::thread_rng();
    let final_theory = final_view.determinize(&HashMap::new(), &mut rng);
    println!("");
    println!("{:?}", final_theory);
}

#[test]
fn test_mcts() {
    let state = GameState::initial(&SAMPLE_DECK);
    let alice_view = state.player_view(Player::Alice);

    let mut alice_mcts = MctsState::new(alice_view, 1.4);

    let mut rng = rand::thread_rng();

    alice_mcts.run_playout(&mut rng);
}
*/
