use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rand::Rng;
use rand::distributions::{IndependentSample, Range};

use hanabi::{Action, ActionResult, GameState};

struct Arrow {
    expected_reward: f64,
    num_samples: f64,
}

impl Arrow {
    fn new() -> Arrow {
        Arrow {
            expected_reward: 0.0,
            num_samples: 0.0,
        }
    }

    fn add_sample(&mut self, reward: f64) {
        let new_expectation = (self.expected_reward * self.num_samples + reward) / (self.num_samples + 1.0);
        self.expected_reward = new_expectation;
        self.num_samples += 1.0;
    }
}

struct Node {
    actions: HashMap<Action, Arrow>,
    total_samples: f64,
}

impl Node {
    fn new() -> Node {
        Node {
            actions: HashMap::new(),
            total_samples: 0.0,
        }
    }

    fn select<R: Rng>(&self, legal_actions: &[Action], exploration: f64, rng: &mut R) -> Action {
        let mut unexplored_actions = Vec::new();
        for &a in legal_actions.iter() {
            if !self.actions.contains_key(&a) {
                unexplored_actions.push(a);
            }
        }
        if unexplored_actions.is_empty() {
            let mut best_actions: Vec<Action> = Vec::new();
            let mut best_grade: f64 = -1.0;

            for &a in legal_actions.iter() {
                let arrow = &self.actions[&a];
                let grade = arrow.expected_reward + exploration * (self.total_samples.ln() / arrow.num_samples).sqrt();
                if grade > best_grade {
                    best_actions.clear();
                    best_actions.push(a);
                    best_grade = grade;
                } else if grade == best_grade {
                    best_actions.push(a);
                }
            }

            let index = Range::new(0, best_actions.len()).ind_sample(rng);

            best_actions[index]
        } else {
            let index = Range::new(0, unexplored_actions.len()).ind_sample(rng);

            unexplored_actions[index]
        }
    }

    fn add_sample(&mut self, action: Action, result: f64) {
        let arrow: &mut Arrow = self.actions.entry(action).or_insert_with(Arrow::new);
        arrow.add_sample(result);
        self.total_samples += 1.0;
    }
}

pub struct MctsState {
    root: GameState,
    exploration: f64,
    nodes: HashMap<u64, Node>,
}

impl MctsState {
    pub fn new(root: GameState, exploration: f64) -> MctsState {
        MctsState {
            root,
            exploration,
            nodes: HashMap::new(),
        }
    }

    pub fn choose_action<R: Rng>(&self, rng: &mut R) -> Action {
        let mut best_actions = Vec::new();
        let mut best_expectation = -1.0;
        let root_fingerprint = self.root.fingerprint();
        let mut hasher = DefaultHasher::new();
        root_fingerprint.hash(&mut hasher);
        let hash = hasher.finish();
        let root_node = self.nodes.get(&hash).unwrap();
        for (&action, arrow) in root_node.actions.iter() {
            if arrow.expected_reward > best_expectation {
                best_actions.clear();
                best_actions.push(action);
                best_expectation = arrow.expected_reward;
            } else if arrow.expected_reward == best_expectation {
                best_actions.push(action);
            }
        }
        let index = Range::new(0, best_actions.len()).ind_sample(rng);
        best_actions[index]
    }

    pub fn run_playout<R: Rng>(&self, rng: &mut R) -> (Vec<(u64, Action)>, f64) {
        let mut updates: Vec<(u64, Action)> = Vec::new();
        let mut current_state = self.root.clone();
        let result = loop {
            let fingerprint = current_state.fingerprint();
            let mut hasher = DefaultHasher::new();
            fingerprint.hash(&mut hasher);
            let hash = hasher.finish();
            let action = match self.nodes.get(&hash) {
                Some(node) => {
                    node.select(&current_state.legal_actions(), self.exploration, rng)
                },
                None => {
                    let legal_actions = current_state.legal_actions();
                    let index = Range::new(0, legal_actions.len()).ind_sample(rng);
                    legal_actions[index]
                },
            };

            updates.push((hash, action));
            
            current_state.determinize(&HashMap::new(), rng);

            match current_state.act(action) {
                ActionResult::Acted(_) => {
                    current_state.reduce_to_current_view();
                },
                ActionResult::Illegal(_) => {
                    panic!("MCTS tried to play an illegal action!");
                },
                ActionResult::Error(_) => {
                    panic!("MCTS encountered an action error!");
                },
                ActionResult::Finished(score) => {
                    break (score as f64);
                },
            }
        };

        (updates, result)
    }

    pub fn update(&mut self, updates: Vec<(u64, Action)>, result: f64) {
        for (hash, action) in updates {
            let node = self.nodes.entry(hash).or_insert_with(Node::new);
            node.add_sample(action, result);
        }
    }
}
