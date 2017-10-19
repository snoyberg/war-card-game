extern crate rand;

use rand::{Rng, SeedableRng, StdRng};
use std::cmp::{Ord, Ordering};
use std::collections::VecDeque;

const MAX_MOVES: usize = 1000000;
const SUITS_PER_PLAYER: usize = 256;

#[derive(Debug, PartialEq)]
struct Deck(VecDeque<u8>);
impl Deck {
    fn new_half_deck() -> Self {
        let mut deck = VecDeque::new();
        for _ in 0..SUITS_PER_PLAYER {
            for i in 2..15 {
                deck.push_back(i);
            }
        }
        Deck(deck)
    }

    fn new_shuffle(rng: &mut StdRng) -> Self {
        let mut deck = Self::new_half_deck();
        rng.shuffle(deck.0.as_mut_slices().0);
        deck
    }

    fn new_empty() -> Self {
        Deck(VecDeque::new())
    }

    #[cfg(test)]
    fn from_vec(vec: Vec<u8>) -> Self {
        Deck(From::from(vec))
    }

    fn draw(&mut self) -> Option<u8> {
        self.0.pop_front()
    }

    fn add(&mut self, card: u8) {
        self.0.push_back(card);
    }

    fn add_pile(&mut self, pile: Deck) {
        for x in pile.0 {
            self.add(x);
        }
    }
}

#[derive(Debug, PartialEq)]
enum GameStepped {
    Cont(GameState),
    Done(Score),
}

#[derive(Debug, PartialEq)]
struct GameState {
    computer: Deck,
    player: Deck,
    moves: usize,
}
impl GameState {
    fn new(mut rng: &mut StdRng) -> Self {
        GameState {
            computer: Deck::new_half_deck(),
            player: Deck::new_shuffle(&mut rng),
            moves: 0,
        }
    }

    fn step(mut self) -> GameStepped {
        use Score::*;
        use GameStepped::*;
        if self.moves >= MAX_MOVES {
            assert!(self.moves == MAX_MOVES);
            return Done(FinishWith(self.player.0.len()));
        }

        let mut computer_pile = Deck::new_empty();
        let mut player_pile = Deck::new_empty();


        loop {
            let (computer, player) =
                match (self.computer.draw(), self.player.draw()) {
                    (None, None) => return Done(TiedAt(self.moves)),
                    (None, Some(_)) => return Done(WinAfter(self.moves)),
                    (Some(_), None) => return Done(LoseAfter(self.moves)),
                    (Some(x), Some(y)) => (x, y)
                };

            computer_pile.add(computer);
            player_pile.add(player);

            match computer.cmp(&player) {
                // player wins
                Ordering::Less => {
                    self.player.add_pile(player_pile);
                    self.player.add_pile(computer_pile);
                    self.moves += 1;
                    return Cont(self);
                }

                // computer wins
                Ordering::Greater => {
                    self.computer.add_pile(computer_pile);
                    self.computer.add_pile(player_pile);
                    self.moves += 1;
                    return Cont(self);
                }

                Ordering::Equal => {
                    for _ in 1..4 {
                        match self.computer.draw() {
                            None => (),
                            Some(x) => computer_pile.add(x),
                        }
                        match self.player.draw() {
                            None => (),
                            Some(x) => player_pile.add(x),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Score {
    WinAfter(usize),
    LoseAfter(usize),
    FinishWith(usize),
    TiedAt(usize),
}

impl Score {
    fn to_int(&self) -> usize {
        match self {
            &Score::LoseAfter(moves) => moves,
            &Score::TiedAt(_moves) => MAX_MOVES + (13 * SUITS_PER_PLAYER),
            &Score::FinishWith(cards) => MAX_MOVES + cards,
            &Score::WinAfter(moves) => MAX_MOVES + (13 * SUITS_PER_PLAYER * 2) + (MAX_MOVES - moves),
        }
    }
}

fn play_game(mut game_state: GameState) -> Score {
    loop {
        game_state = match game_state.step() {
            GameStepped::Cont(game_state) => game_state,
            GameStepped::Done(score) => return score,
        }
    }
}

fn main() {
    for x in 1..1001 {
        let seed: &[_] = &[x];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let score = play_game(GameState::new(&mut rng));
        println!("{}: {} ({:?})", x, score.to_int(), score);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Score::*;
    use GameStepped::*;

    #[test]
    fn empty_computer() {
        let gs = GameState {
            computer: Deck::from_vec(vec![]),
            player: Deck::from_vec(vec![2]),
            moves: 0,
        };

        assert_eq!(gs.step(), Done(WinAfter(0)));
    }

    #[test]
    fn empty_player() {
        let gs = GameState {
            computer: Deck::from_vec(vec![2]),
            player: Deck::from_vec(vec![]),
            moves: 0,
        };

        assert_eq!(gs.step(), Done(LoseAfter(0)));
    }

    #[test]
    fn empty_tied_war() {
        let gs = GameState {
            computer: Deck::from_vec(vec![2, 14, 14, 14, 2]),
            player: Deck::from_vec(vec![2, 2, 2, 2, 2]),
            moves: 2,
        };

        assert_eq!(gs.step(), Done(TiedAt(2)));
    }

    #[test]
    fn player_trick() {
        let gs1 = GameState {
            computer: Deck::from_vec(vec![2, 3]),
            player: Deck::from_vec(vec![4, 5]),
            moves: 6,
        };
        let gs2 = GameState {
            computer: Deck::from_vec(vec![3]),
            player: Deck::from_vec(vec![5, 4, 2]),
            moves: 7,
        };

        assert_eq!(gs1.step(), Cont(gs2));
    }

    #[test]
    fn computer_trick() {
        let gs1 = GameState {
            player: Deck::from_vec(vec![2, 3]),
            computer: Deck::from_vec(vec![4, 5]),
            moves: 6,
        };
        let gs2 = GameState {
            player: Deck::from_vec(vec![3]),
            computer: Deck::from_vec(vec![5, 4, 2]),
            moves: 7,
        };

        assert_eq!(gs1.step(), Cont(gs2));
    }

    #[test]
    fn war() {
        let gs1 = GameState {
            player: Deck::from_vec(vec![2, 3, 4, 5, 6, 7]),
            computer: Deck::from_vec(vec![2, 8, 9, 10, 11]),
            moves: 8,
        };
        let gs2 = GameState {
            player: Deck::from_vec(vec![7]),
            computer: Deck::from_vec(vec![2, 8, 9, 10, 11, 2, 3, 4, 5, 6]),
            moves: 9,
        };

        assert_eq!(gs1.step(), Cont(gs2));
    }
}
