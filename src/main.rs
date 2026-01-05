use rand::distr::weighted::WeightedIndex;
use rand::{prelude::*, rng};
use rayon::prelude::*;
use std::cmp::min;
use std::io::{self, Write};
use std::num::ParseIntError;

#[derive(Debug, Clone, Copy)]
pub enum Die {
    Left,
    Right,
    Center,
    Dot,
}

#[derive(Debug, Clone)]
struct Game {
    players: Vec<u8>,
}

enum GameState {
    Normal,
    Winner(usize),
}

impl Game {
    pub fn new(no_of_players: usize) -> Self {
        let mut players = Vec::with_capacity(no_of_players);
        for _i in 0..no_of_players {
            players.push(3)
        }
        Game { players }
    }
    fn pass_left(&mut self, active_player: usize) {
        let left_player = {
            if active_player == 0 {
                self.players.len() - 1
            } else {
                active_player - 1
            }
        };
        self.players[left_player] += 1;
        self.players[active_player] -= 1;
    }
    fn pass_right(&mut self, active_player: usize) {
        let right_player = {
            if active_player == self.players.len() - 1 {
                0
            } else {
                active_player + 1
            }
        };
        self.players[right_player] += 1;
        self.players[active_player] -= 1;
    }
    fn is_there_winner(&self) -> GameState {
        let mut possible_winner = self.players.len();
        let mut qty_players_with_chips = 0;
        for (player, chips) in self.players.iter().enumerate() {
            if *chips > 0 {
                possible_winner = player;
                qty_players_with_chips += 1
            }
        }
        if qty_players_with_chips == 1 {
            GameState::Winner(possible_winner)
        } else {
            GameState::Normal
        }
    }
    pub fn play(mut self, prints: bool) -> usize {
        let no_of_players = self.players.len().clone();
        loop {
            for active_player in 0..no_of_players {
                let chips = self.players[active_player].clone();
                if prints {
                    print!("\nplayer {active_player}'s turn with {chips} chips");
                }
                if chips == 0u8 {
                    continue;
                }

                let dice_to_roll = min(chips, 3);
                let mut all_dots = true;
                if prints {
                    print!(" rolled");
                }
                for _ in 0..dice_to_roll {
                    match Die::roll() {
                        Die::Left => {
                            if prints {
                                print!(" left");
                            }
                            self.pass_left(active_player);
                            all_dots = false;
                        }
                        Die::Right => {
                            if prints {
                                print!(" right");
                            }
                            self.pass_right(active_player);
                            all_dots = false;
                        }
                        Die::Center => {
                            if prints {
                                print!(" center");
                            }
                            self.players[active_player] -= 1;
                            all_dots = false;
                        }
                        Die::Dot => {
                            if prints {
                                print!(" dot")
                            }
                        }
                    }
                }
                if !all_dots {
                    match self.is_there_winner() {
                        GameState::Winner(winner) => return winner,
                        GameState::Normal => {}
                    }
                }
            }
        }
    }
}
impl Die {
    pub fn roll() -> Self {
        let weights = [1, 1, 1, 3];
        let dist =
            WeightedIndex::new(&weights).expect("weights should be non-negative and non-empty");
        let mut myrng = rng();
        match dist.sample(&mut myrng) {
            0 => Die::Left,
            1 => Die::Right,
            2 => Die::Center,
            3 => Die::Dot,
            _ => unreachable!(),
        }
    }
}

fn main() {
    print!("how many games to simulate? ");
    io::stdout().flush().unwrap();
    let mut entry = String::new();

    io::stdin().read_line(&mut entry).unwrap();

    let games = match entry.trim().parse() {
        Ok(games) => games,
        Err(_) => {
            print!("\ndidn't understand input, using 1,000,000\n");
            1_000_000
        }
    };
    print!("how many players per game? ");
    io::stdout().flush().unwrap();
    let mut entry = String::new();
    io::stdin().read_line(&mut entry).unwrap();
    let no_of_players: Result<usize, ParseIntError> = entry.trim().parse();
    let no_of_players = no_of_players.unwrap();
    let to_print = games == 1;
    let player_wins = (0..games)
        .into_par_iter()
        // create a “zeroed” local count array for each thread
        .fold(
            || vec![0usize; no_of_players],
            |mut local_counts, _| {
                let game = Game::new(no_of_players);
                let winner = game.play(to_print);
                local_counts[winner] += 1;
                local_counts
            },
        )
        // reduce: merge two local count arrays into one
        .reduce(
            || vec![0usize; no_of_players],
            |mut acc, local_counts| {
                for (i, &c) in local_counts.iter().enumerate() {
                    acc[i] += c;
                }
                acc
            },
        );
    let even_perc = 100.0 / no_of_players as f64;
    for (player, wins) in player_wins.into_iter().enumerate() {
        let percent = wins as f64 / games as f64 * 100.0;
        let diff = percent - even_perc;
        println!("player {} win rate biased by {:.5}", player, diff);
    }
}
