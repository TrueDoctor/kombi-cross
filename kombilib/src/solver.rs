use crate::parsing::{self, Crossing};

use crate::output::Square;

use super::Instance;

#[derive(Clone, Debug)]
pub struct State {
    assignments: Vec<Option<std::num::NonZeroU8>>,
    placed_words: u64,
}

pub struct Solver {
    instance: Instance,
    word_lens: Vec<Vec<usize>>,
    box_lens: Vec<Vec<usize>>,
    box_crossings: Vec<Vec<Crossing>>,
}

impl Solver {
    pub fn new(mut instance: Instance) -> Self {
        instance.words.sort_by_key(|x| x.len());
        let max_len = instance.boxes.iter().map(|x| x.len).max().unwrap_or(0) + 1;
        let mut word_lens = vec![vec![]; max_len];
        let mut box_lens = vec![vec![]; max_len];
        let mut box_crossings = vec![vec![]; instance.boxes.len()];

        for (i, word) in instance.words.iter().enumerate() {
            word_lens[word.len()].push(i);
        }

        for (i, cross_box) in instance.boxes.iter().enumerate() {
            box_lens[cross_box.len].push(i);
        }

        for crossing in instance.crossings.iter() {
            box_crossings[crossing.id_a].push(*crossing);
            box_crossings[crossing.id_b].push(crossing.reverse());
        }

        Self {
            instance,
            word_lens,
            box_lens,
            box_crossings,
        }
    }
    pub fn solve(&self) -> Vec<State> {
        let mut queue = vec![State {
            assignments: vec![None; self.instance.boxes.len()],
            placed_words: 0,
        }];
        let mut solutions = Vec::new();

        while let Some(state) = queue.pop() {
            if state.placed_words.count_ones() as usize == self.instance.words.len() {
                solutions.push(state);
            } else {
                self.next_states(&state, &mut queue);
            }
        }

        solutions
    }

    pub fn consensus(&self, solutions: &[State]) -> State {
        let mut consensus = solutions[0].clone();
        for (i, block) in consensus.assignments.iter_mut().enumerate() {
            if solutions.iter().any(|s| s.assignments[i] != *block) {
                *block = None;
            }
        }
        consensus
    }

    pub fn next_states(&self, state: &State, queue: &mut Vec<State>) {
        let Some(group) = self.best_group(state) else {
            return;
        };

        let Some(&word) = self.word_lens[group]
            .iter()
            .find(|w_id| state.placed_words & (1 << *w_id) == 0)
        else {
            eprintln!("did not find word to place");
            return;
        };
        self.place_word(state, group, word, queue);
    }

    fn place_word(&self, state: &State, group: usize, word: usize, queue: &mut Vec<State>) {
        for &slot in &self.box_lens[group] {
            if state.assignments[slot].is_some() {
                continue;
            }
            if self.valid_move(state, word, slot) {
                let mut new_state = state.clone();
                new_state.assignments[slot] = Some((word as u8 + 1).try_into().unwrap());
                new_state.placed_words |= 1 << word;
                queue.push(new_state);
            }
        }
    }

    fn valid_move(&self, state: &State, new_word: usize, new_box: usize) -> bool {
        self.box_crossings[new_box].iter().all(|c| {
            let Some(other_word) = state.assignments[c.id_b] else {
                return true;
            };
            assert_eq!(
                self.instance.boxes[new_box].len,
                self.instance.words[new_word].len()
            );

            // safe as bytes: `umlaut()` folds every word down to ASCII
            self.instance.words[new_word].as_bytes()[c.c_a as usize]
                == self.instance.words[other_word.get() as usize - 1].as_bytes()[c.c_b as usize]
        })
    }

    pub fn best_group(&self, state: &State) -> Option<usize> {
        let mut min_i = None;
        let mut min_perms = usize::MAX;
        for (i, (words, boxes)) in self.word_lens.iter().zip(&self.box_lens).enumerate() {
            let remaining_words = words
                .iter()
                .filter(|w_id| state.placed_words & (1 << **w_id) == 0)
                .count();
            let remaining_boxes = boxes
                .iter()
                .filter(|b_id| state.assignments[**b_id].is_none())
                .count();
            if remaining_words == 0 {
                continue;
            }

            // more words than slots: this branch can never complete
            if remaining_words > remaining_boxes {
                return None;
            }

            let ops = options(remaining_boxes, remaining_words);
            if ops < min_perms {
                min_perms = ops;
                min_i = Some(i);
            }
        }
        min_i
    }

    pub fn format_state(&self, state: &State) -> Vec<Square> {
        self.instance
            .boxes
            .iter()
            .enumerate()
            .flat_map(|(b_i, b)| {
                (0..b.len).map(move |i| {
                    let char = state.assignments[b_i]
                        .and_then(|w| self.instance.words[w.get() as usize - 1].chars().nth(i))
                        .unwrap_or(' ');
                    let i = i as i32;
                    let (x, y) = match b.dir {
                        parsing::Direction::Left => (b.x - i, b.y),
                        parsing::Direction::Right => (b.x + i, b.y),
                        parsing::Direction::Up => (b.x, b.y - i),
                        parsing::Direction::Down => (b.x, b.y + i),
                    };
                    Square {
                        x,
                        y,
                        char: char.to_string().to_ascii_uppercase(),
                        dir: (if i == 0 { Some(b.dir.clone()) } else { None }),
                        len: (if i == 0 { Some(b.len) } else { None }),
                    }
                })
            })
            .collect()
    }
}

fn options(n: usize, k: usize) -> usize {
    debug_assert!(k <= n);
    ((n + 1 - k)..=n).product()
}
