use rand::distr::StandardUniform;
use rand::prelude::*;
use rayon::prelude::*;
use std::cmp::min;
use std::io::{self, Write};
use std::num::ParseIntError;
const MAX_DICE: u8 = 3;
#[derive(Debug, Clone, Copy)]
pub enum Die {
    Left,
    Right,
    Center,
    Dot,
}
impl Distribution<Die> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Die {
        let index: u8 = rng.random_range(0..6);
        match index {
            0 => Die::Left,
            1 => Die::Right,
            2 => Die::Center,
            _ => Die::Dot,
        }
    }
}

struct Game<R: RngCore> {
    players: Vec<u8>,
    rng: R,
    prints: bool,
}
impl<R: RngCore> Game<R> {
    pub fn new(no_of_players: usize, rng: R, prints: bool) -> Self {
        Game {
            players: vec![MAX_DICE; no_of_players],
            rng,
            prints,
        }
    }
    fn print_turn(&self, chips: &u8, active_player: &usize) {
        if !self.prints {
            return;
        }

        print!("\nplayer {active_player}'s turn with {chips} chips");
        if chips > &0 {
            print!(" rolled");
        }
    }
    fn pass(&mut self, all_dots: &mut bool, active_player: usize, die: Die) {
        let (res, dot) = match die {
            Die::Left => {
                self.pass_left(active_player);

                (" left", false)
            }
            Die::Right => {
                self.pass_right(active_player);

                (" right", false)
            }
            Die::Center => {
                self.players[active_player] -= 1;

                (" center", false)
            }
            Die::Dot => (" dot", true),
        };
        *all_dots = *all_dots && dot;
        if self.prints {
            print!("{res}");
        }
    }
    fn pass_left(&mut self, active_player: usize) {
        let n = self.players.len();
        let left_player = (active_player + n - 1) % n;
        self.players[left_player] += 1;
        self.players[active_player] -= 1;
    }
    fn pass_right(&mut self, active_player: usize) {
        let n = self.players.len();
        let right_player = (active_player + 1) % n;
        self.players[right_player] += 1;
        self.players[active_player] -= 1;
    }
    fn print_ending(&self, active_player: usize) {
        if !self.prints {
            return;
        }
        let chips = self.players[active_player];
        print!(". Ending with {chips} chips");
    }
    fn is_there_winner(&self) -> GameState {
        let mut possible_winner = self.players.len();
        let mut qty_players_with_chips = 0u8;
        for (player, chips) in self.players.iter().enumerate() {
            if *chips > 0 {
                possible_winner = player;
                qty_players_with_chips += 1
            }
            if qty_players_with_chips > 1 {
                break;
            }
        }
        if qty_players_with_chips == 1 {
            if self.prints {
                println!();
            }
            GameState::Winner(possible_winner)
        } else {
            GameState::Normal
        }
    }
    pub fn play(mut self) -> usize {
        let no_of_players = self.players.len();
        loop {
            for active_player in 0..no_of_players {
                let chips = self.players[active_player];
                self.print_turn(&chips, &active_player);
                if chips == 0u8 {
                    continue;
                }
                let dice_to_roll = min(chips, MAX_DICE);
                let mut all_dots = true;
                for _ in 0..dice_to_roll {
                    let die: Die = self.rng.random();
                    self.pass(&mut all_dots, active_player, die);
                }
                self.print_ending(active_player);

                if !all_dots {
                    // if player gets all dots, no need to check for winner
                    match self.is_there_winner() {
                        GameState::Winner(winner) => return winner,
                        GameState::Normal => {}
                    }
                }
            }
        }
    }
}

enum GameState {
    Normal,
    Winner(usize),
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
    let no_of_playerss: Vec<usize> = match entry.contains(',') {
        true => entry
            .split(',')
            .map(|ent| {
                let no_of_players: Result<usize, ParseIntError> = ent.trim().parse();
                no_of_players.unwrap()
            })
            .collect(),
        false => {
            let no_of_players: Result<usize, ParseIntError> = entry.trim().parse();
            vec![no_of_players.unwrap()]
        }
    };

    let to_print = games == 1;
    let vec_players_size = &no_of_playerss.len();
    for no_of_players in no_of_playerss {
        if *vec_players_size > 1 {
            println!("with {no_of_players} players");
        }
        let player_wins = (0..games)
            .into_par_iter()
            .map_init(rand::rng, |rng, _| {
                let game = Game::new(no_of_players, rng, to_print);
                game.play()
            })
            .fold(
                || vec![0usize; no_of_players],
                |mut local_counts, x| {
                    local_counts[x] += 1;
                    local_counts
                },
            )
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
            println!("player {} won {wins}, rate biased by {:.5}", player, diff);
        }
    }
}
