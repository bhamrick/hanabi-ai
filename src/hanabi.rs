// For now, we are going to write assuming four player, no variant.
// Generalizations will be possible in the future.

use std::collections::HashMap;
use std::ops::AddAssign;
use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice};

use determinization::determinize;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    Alice,
    Bob,
    Cathy,
    Dave,
}

impl Player {
    pub fn next(self) -> Player {
        match self {
            Player::Alice => Player::Bob,
            Player::Bob => Player::Cathy,
            Player::Cathy => Player::Dave,
            Player::Dave => Player::Alice,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl Rank {
    pub fn playable_on(self, pile: Option<Rank>) -> bool {
        match pile {
            None => self == Rank::One,
            Some(Rank::One) => self == Rank::Two,
            Some(Rank::Two) => self == Rank::Three,
            Some(Rank::Three) => self == Rank::Four,
            Some(Rank::Four) => self == Rank::Five,
            Some(Rank::Five) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

pub fn deck_distribution() -> HashMap<Card, usize> {
    let mut dist = HashMap::new();

    for s in [Suit::Red, Suit::Green, Suit::Blue, Suit::Yellow, Suit::Purple].iter().cloned() {
        for r in [Rank::One, Rank::Two, Rank::Three, Rank::Four, Rank::Five].iter().cloned() {
            let c = Card { suit: s, rank: r };
            let count = match r {
                Rank::One => 3,
                Rank::Two => 2,
                Rank::Three => 2,
                Rank::Four => 2,
                Rank::Five => 1,
            };
            dist.insert(c, count);
        }
    }

    dist
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Clue {
    Suit(Suit),
    Rank(Rank),
}

impl Clue {
    pub fn matches(&self, card: Card) -> bool {
        match self {
            &Clue::Suit(s) => {
                s == card.suit
            },
            &Clue::Rank(r) => {
                r == card.rank
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct CardId(pub usize);

impl CardId {
    pub fn increment(&mut self) {
        let &mut CardId(ref mut id) = self;
        *id += 1;
    }
}

// GameState encompasses both the omniscient state and the view for each player,
// with the only difference being which cards are included in card_map.
#[derive(Debug, Clone)]
pub struct GameState {
    card_map: HashMap<CardId, Card>,
    deck_size: usize,
    next_card_id: CardId,
    current_turn: Player,
    final_turn: Option<Player>,
    hands: HashMap<Player, Vec<CardId>>,
    played_cards: Vec<CardId>,
    discarded_cards: Vec<CardId>,
    piles: HashMap<Suit, Rank>,
    clues: u8,
    strikes: u8,
    information: HashMap<CardId, Vec<Information>>,
    action_log: Vec<CompletedAction>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Discard(usize),
    Play(usize),
    Clue(Player, Clue),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Information(Clue, bool);

impl Information {
    pub fn consistent_with(self, card: Card) -> bool {
        let Information(clue, matches) = self;
        matches == clue.matches(card)
    }
}

// A description of an action plus the information revealed by that action.
// For discards and plays, the information is the identity of the card.
// For clues, the information is the set of cards that matched it.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CompletedAction {
    Discarded(usize, Card),
    Played(usize, Card),
    Clued(Player, Clue, Vec<CardId>),
}

// Game states are not hashable, but each one is uniquely determined by the
// sequence of actions taken (and the information revealed by them), as well
// as the set of visible cards.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Fingerprint {
    known_cards: Vec<(CardId, Card)>,
    actions: Vec<CompletedAction>,
}

#[derive(Debug, Clone)]
pub enum IllegalAction {
    NoSuchCard,
    NoMatchingCards,
    CluedSelf,
    NoClues,
    TooManyClues,
}

#[derive(Debug, Clone)]
pub enum ActionError {
    UnknownCard,
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    Acted(CompletedAction),
    Illegal(IllegalAction),
    Error(ActionError),
    Finished(i8),
}

impl GameState {
    pub fn initial(deck_order: &[Card]) -> GameState {
        let mut card_map = HashMap::new();
        for (i, c) in deck_order.iter().cloned().enumerate() {
            card_map.insert(CardId(i), c);
        }

        GameState {
            card_map,
            deck_size: deck_order.len() - 16,
            next_card_id: CardId(16),
            current_turn: Player::Alice,
            final_turn: None,
            hands: vec![
                (Player::Alice, vec![CardId(0), CardId(4), CardId(8), CardId(12)]),
                (Player::Bob, vec![CardId(1), CardId(5), CardId(9), CardId(13)]),
                (Player::Cathy, vec![CardId(2), CardId(6), CardId(10), CardId(14)]),
                (Player::Dave, vec![CardId(3), CardId(7), CardId(11), CardId(15)]),
            ].into_iter().collect(),
            played_cards: Vec::new(),
            discarded_cards: Vec::new(),
            piles: HashMap::new(),
            clues: 8,
            strikes: 0,
            information: HashMap::new(),
            action_log: Vec::new(),
        }
    }

    pub fn reduce_to_player_view(&mut self, player: Player) {
        let mut viewed_card_map = HashMap::new();
        for (&p, h) in self.hands.iter() {
            if p != player {
                for c_id in h.iter() {
                    if let Some(&c) = self.card_map.get(c_id) {
                        viewed_card_map.insert(*c_id, c);
                    }
                }
            }
        }

        for c_id in self.played_cards.iter() {
            if let Some(&c) = self.card_map.get(c_id) {
                viewed_card_map.insert(*c_id, c);
            }
        }

        for c_id in self.discarded_cards.iter() {
            if let Some(&c) = self.card_map.get(c_id) {
                viewed_card_map.insert(*c_id, c);
            }
        }

        self.card_map = viewed_card_map;
    }

    pub fn reduce_to_current_view(&mut self) {
        let player = self.current_turn;
        self.reduce_to_player_view(player);
    }

    pub fn player_view(&self, player: Player) -> GameState {
        let mut viewed_card_map = HashMap::new();
        for (&p, h) in self.hands.iter() {
            if p != player {
                for c_id in h.iter() {
                    if let Some(&c) = self.card_map.get(c_id) {
                        viewed_card_map.insert(*c_id, c);
                    }
                }
            }
        }

        for c_id in self.played_cards.iter() {
            if let Some(&c) = self.card_map.get(c_id) {
                viewed_card_map.insert(*c_id, c);
            }
        }

        for c_id in self.discarded_cards.iter() {
            if let Some(&c) = self.card_map.get(c_id) {
                viewed_card_map.insert(*c_id, c);
            }
        }

        GameState {
            card_map: viewed_card_map,
            deck_size: self.deck_size,
            next_card_id: self.next_card_id,
            current_turn: self.current_turn,
            final_turn: self.final_turn,
            hands: self.hands.clone(),
            played_cards: self.played_cards.clone(),
            discarded_cards: self.discarded_cards.clone(),
            piles: self.piles.clone(),
            clues: self.clues,
            strikes: self.strikes,
            information: self.information.clone(),
            action_log: self.action_log.clone(),
        }
    }

    pub fn act(&mut self, action: Action) -> ActionResult {
        let current_player = self.current_turn;
        match action {
            Action::Discard(i) => {
                if i >= self.hands[&current_player].len() {
                    return ActionResult::Illegal(IllegalAction::NoSuchCard);
                }
                if self.clues == 8 {
                    return ActionResult::Illegal(IllegalAction::TooManyClues);
                }

                let c_id: CardId = self.hands[&current_player][i];
                let c = match self.card_map.get(&c_id) {
                    Some(&c) => c,
                    None => return ActionResult::Error(ActionError::UnknownCard),
                };

                self.hands.get_mut(&current_player).unwrap().remove(i);
                self.discarded_cards.push(c_id);
                self.information.remove(&c_id);
                self.clues += 1;

                if self.final_turn == Some(current_player) {
                    let mut total_score = 0;
                    for (_, &r) in self.piles.iter() {
                        total_score += match r {
                            Rank::One => 1,
                            Rank::Two => 2,
                            Rank::Three => 3,
                            Rank::Four => 4,
                            Rank::Five => 5,
                        };
                    }
                    return ActionResult::Finished(total_score);
                }

                if self.deck_size > 0 {
                    self.hands.get_mut(&current_player).unwrap().push(self.next_card_id);
                    self.next_card_id.increment();
                    self.deck_size -= 1;
                    
                    if self.deck_size == 0 {
                        self.final_turn = Some(current_player);
                    }
                }

                self.current_turn = current_player.next();

                let completed_action = CompletedAction::Discarded(i, c);

                self.action_log.push(completed_action.clone());
                ActionResult::Acted(completed_action)
            },
            Action::Play(i) => {
                if i >= self.hands[&current_player].len() {
                    return ActionResult::Illegal(IllegalAction::NoSuchCard);
                }

                let c_id: CardId = self.hands[&current_player][i];
                let c = match self.card_map.get(&c_id) {
                    Some(&c) => c,
                    None => return ActionResult::Error(ActionError::UnknownCard),
                };

                self.hands.get_mut(&current_player).unwrap().remove(i);
                self.information.remove(&c_id);

                let suit_pile: Option<Rank> = self.piles.get(&c.suit).cloned();
                if c.rank.playable_on(suit_pile) {
                    self.piles.insert(c.suit, c.rank);
                    self.played_cards.push(c_id);

                    if c.rank == Rank::Five && self.clues < 8 {
                        self.clues += 1;
                    }
                } else {
                    self.discarded_cards.push(c_id);
                    self.strikes += 1;

                    if self.strikes == 3 {
                        return ActionResult::Finished(0);
                    }
                }

                if self.final_turn == Some(current_player) {
                    let mut total_score = 0;
                    for (_, &r) in self.piles.iter() {
                        total_score += match r {
                            Rank::One => 1,
                            Rank::Two => 2,
                            Rank::Three => 3,
                            Rank::Four => 4,
                            Rank::Five => 5,
                        };
                    }
                    return ActionResult::Finished(total_score);
                }

                if self.deck_size > 0 {
                    self.hands.get_mut(&current_player).unwrap().push(self.next_card_id);
                    self.next_card_id.increment();
                    self.deck_size -= 1;

                    if self.deck_size == 0 {
                        self.final_turn = Some(current_player);
                    }
                }

                self.current_turn = current_player.next();
                let completed_action = CompletedAction::Played(i, c);
                self.action_log.push(completed_action.clone());

                ActionResult::Acted(completed_action)
            },
            Action::Clue(target, clue) => {
                if target == self.current_turn {
                    return ActionResult::Illegal(IllegalAction::CluedSelf);
                }
                if self.clues == 0 {
                    return ActionResult::Illegal(IllegalAction::NoClues);
                }

                let mut matching_cards = Vec::new();
                for &c_id in self.hands[&target].iter() {
                    let c = match self.card_map.get(&c_id) {
                        Some(&c) => c,
                        None => return ActionResult::Error(ActionError::UnknownCard),
                    };
                    if clue.matches(c) {
                        matching_cards.push(c_id);
                    }
                    let info_vec: &mut Vec<Information> = self.information.entry(c_id).or_insert_with(Vec::new);
                    info_vec.push(Information(clue, clue.matches(c)));
                }

                if matching_cards.is_empty() {
                    return ActionResult::Illegal(IllegalAction::NoMatchingCards);
                }

                self.clues -= 1;

                if self.final_turn == Some(current_player) {
                    let mut total_score = 0;
                    for (_, &r) in self.piles.iter() {
                        total_score += match r {
                            Rank::One => 1,
                            Rank::Two => 2,
                            Rank::Three => 3,
                            Rank::Four => 4,
                            Rank::Five => 5,
                        };
                    }
                    return ActionResult::Finished(total_score);
                }

                self.current_turn = current_player.next();

                let completed_action = CompletedAction::Clued(target, clue, matching_cards);
                self.action_log.push(completed_action.clone());
                ActionResult::Acted(completed_action)
            },
        }
    }

    pub fn unknown_cards(&self) -> Vec<CardId> {
        let mut unknowns = Vec::new();

        for (_, h) in self.hands.iter() {
            for &c_id in h.iter() {
                if !self.card_map.contains_key(&c_id) {
                    unknowns.push(c_id);
                }
            }
        }

        for i in 0..self.deck_size {
            let CardId(base) = self.next_card_id;
            let c_id = CardId(base+i);
            if !self.card_map.contains_key(&c_id) {
                unknowns.push(c_id);
            }
        }

        unknowns
    }

    pub fn current_player(&self) -> Player {
        self.current_turn
    }

    pub fn remaining_card_counts(&self) -> HashMap<Card, usize> {
        let mut seen_counts: HashMap<Card, usize> = HashMap::new();
        for (_, &c) in self.card_map.iter() {
            seen_counts.entry(c).or_insert(0).add_assign(1);
        }

        let mut remaining_counts = HashMap::new();
        let all_counts = deck_distribution();
        for (&c, &n) in all_counts.iter() {
            let seen = *seen_counts.get(&c).unwrap_or(&0);
            if seen > n {
                panic!("More seen {:?}s than should exist", c);
            }
            if n > seen {
                remaining_counts.insert(c, n - seen);
            }
        }

        remaining_counts
    }

    pub fn determinize<R>(&mut self, priors: &HashMap<CardId, Vec<Weighted<Card>>>, rng: &mut R)
        where
        R: Rng,
    {
        let unknowns = self.unknown_cards();
        let remaining_counts = self.remaining_card_counts();
        let mut restricted_priors: HashMap<CardId, Vec<Weighted<Card>>> = HashMap::new();
        for &c_id in unknowns.iter() {
            if let Some(card_info) = self.information.get(&c_id) {
                if let Some(original_prior) = priors.get(&c_id) {
                    // Restrict original prior to possible cards.
                    let mut restricted_prior = Vec::new();
                    for w in original_prior.iter() {
                        let mut ok = true;
                        for info in card_info.iter() {
                            if !info.consistent_with(w.item) {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            restricted_prior.push(w.clone());
                        }
                    }
                    restricted_priors.insert(c_id, restricted_prior);
                } else {
                    // Use deck distribution restricted to possible cards.
                    let mut restricted_prior = Vec::new();
                    for (&card, &count) in deck_distribution().iter() {
                        let mut ok = true;
                        for info in card_info.iter() {
                            if !info.consistent_with(card) {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            restricted_prior.push(Weighted {
                                weight: count as u32,
                                item: card,
                            });
                        }
                    }
                    restricted_priors.insert(c_id, restricted_prior);
                }
            }
        }
        for (&c_id, existing_prior) in priors.iter() {
            if !restricted_priors.contains_key(&c_id) {
                restricted_priors.insert(c_id, existing_prior.clone());
            }
        }

        let mut new_priors: HashMap<CardId, WeightedChoice<Card>> = HashMap::new();
        for (&c_id, weights) in restricted_priors.iter_mut() {
            let dist = WeightedChoice::new(weights);
            new_priors.insert(c_id, dist);
        }

        let assignment = determinize(
            &unknowns,
            &remaining_counts,
            &new_priors,
            rng,
        );

        for (&c_id, &c) in assignment.iter() {
            self.card_map.insert(c_id, c);
        }
    }

    pub fn fingerprint(&self) -> Fingerprint {
        let mut known_cards = Vec::new();
        for (&c_id, &c) in self.card_map.iter() {
            known_cards.push((c_id, c));
        }
        known_cards.sort();

        Fingerprint {
            known_cards,
            actions: self.action_log.clone(),
        }
    }

    pub fn current_view(&self) -> GameState {
        self.player_view(self.current_turn)
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        let current_player = self.current_turn;

        let mut actions = Vec::new();

        for (i, _) in self.hands[&current_player].iter().enumerate() {
            if self.clues < 8 {
                actions.push(Action::Discard(i));
            }
            actions.push(Action::Play(i));
        }

        if self.clues > 0 {
            for (&target, target_hand) in self.hands.iter() {
                if target == current_player {
                    continue;
                }
                
                for &suit in [Suit::Red, Suit::Green, Suit::Blue, Suit::Yellow, Suit::Purple].iter() {
                    let suit_clue = Clue::Suit(suit);

                    let mut has_match = false;
                    for c_id in target_hand.iter() {
                        if let Some(&c) = self.card_map.get(c_id) {
                            if suit_clue.matches(c) {
                                has_match = true;
                                break;
                            }
                        }
                    }

                    if has_match {
                        actions.push(Action::Clue(target, suit_clue));
                    }
                }
                
                for &rank in [Rank::One, Rank::Two, Rank::Three, Rank::Four, Rank::Five].iter() {
                    let rank_clue = Clue::Rank(rank);

                    let mut has_match = false;
                    for c_id in target_hand.iter() {
                        if let Some(&c) = self.card_map.get(c_id) {
                            if rank_clue.matches(c) {
                                has_match = true;
                                break;
                            }
                        }
                    }

                    if has_match {
                        actions.push(Action::Clue(target, rank_clue));
                    }
                }
            }
        }

        actions
    }
}
