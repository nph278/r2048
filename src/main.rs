#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::enum_glob_use)]

use std::io::stdout;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::style::{Color, Stylize};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};

use rand::prelude::*;

type Board = Vec<Vec<Option<u16>>>;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

use Direction::*;

fn add_random(board: &mut Board, score: &mut usize, rng: &mut ThreadRng, bx: usize, by: usize) {
    let mut empty_indices = Vec::new();

    for i in 0..by {
        for j in 0..bx {
            if board[i][j].is_none() {
                empty_indices.push((i, j));
            }
        }
    }

    if let Some((i, j)) = empty_indices.choose(rng) {
        let n = if rng.gen_bool(0.75) { 2 } else { 4 };
        board[*i][*j] = Some(n);
        *score += n as usize;
    }
}

fn render(board: &Board, score: usize) {
    execute!(stdout(), MoveTo(1, 1)).unwrap();
    print!("{}", score.to_string().with(Color::Green));

    for (r, row) in board.iter().enumerate() {
        for (i, n) in row.iter().enumerate() {
            let mut s = n.map(|x| x.to_string()).unwrap_or(String::new());
            while s.len() < 4 {
                s = format!(" {}", s);
            }

            let color = match n {
                None => Color::Black,
                Some(2) => Color::Grey,
                Some(4) => Color::Green,
                Some(8) => Color::Blue,
                Some(16) => Color::Yellow,
                Some(32) => Color::Red,
                Some(64) => Color::Magenta,
                Some(128) => Color::Cyan,
                Some(2048) => Color::DarkYellow,
                _ => Color::Cyan,
            };

            execute!(stdout(), MoveTo(i as u16 * 3 + 1, r as u16 * 3 + 2)).unwrap();
            print!("{}", &s[0..2].with(color));

            execute!(stdout(), MoveTo(i as u16 * 3 + 1, r as u16 * 3 + 3)).unwrap();
            print!("{}", &s[2..4].with(color));
        }
    }
}

fn adj_indices(x: usize, y: usize, d: Direction, bx: usize, by: usize) -> Option<(usize, usize)> {
    let x = x as isize;
    let y = y as isize;
    let (x, y) = match d {
        Up => (x, y - 1),
        Down => (x, y + 1),
        Left => (x - 1, y),
        Right => (x + 1, y),
    };

    if x >= 0 && y >= 0 && x < bx as isize && y < by as isize {
        Some((x as usize, y as usize))
    } else {
        None
    }
}

fn shift(board: &mut Board, score: &mut usize, d: Direction, bx: usize, by: usize) {
    for i in 0..by {
        for j in 0..bx {
            match board[i][j] {
                Some(r) => {
                    if let Some((x, y)) = adj_indices(i, j, d, bx, by) {
                        if let Some(n) = board[x][y] {
                            if n == r {
                                board[x][y] = None;
                                board[i][j] = Some(n * 2);
                                *score += n as usize * 2;
                            }
                        }
                    }
                }
                None => {
                    if let Some((x, y)) = adj_indices(i, j, d, bx, by) {
                        if let Some(n) = board[x][y] {
                            board[x][y] = None;
                            board[i][j] = Some(n);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut rng = thread_rng();
    let mut score = 0;

    let mut bx = 4;

    let mut args = std::env::args();
    args.next();
    if let Some(a) = args.next() {
        if let Ok(a) = a.parse() {
            bx = a;
        }
    }

    let mut board: Board = vec![vec![None; bx]; bx];
    add_random(&mut board, &mut score, &mut rng, bx, bx);

    enable_raw_mode().unwrap();
    execute!(stdout(), Hide).unwrap();
    execute!(stdout(), Clear(ClearType::All)).unwrap();

    loop {
        render(&board, score);
        match read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => break,
            Event::Key(KeyEvent {
                code:
                    code @ (KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Char('h' | 'j' | 'k' | 'l')),
                ..
            }) => {
                let d = match code {
                    KeyCode::Up | KeyCode::Char('k') => Right,
                    KeyCode::Down | KeyCode::Char('j') => Left,
                    KeyCode::Left | KeyCode::Char('h') => Down,
                    KeyCode::Right | KeyCode::Char('l') => Up,
                    _ => panic!(),
                };

                for _ in 0..bx - 1 {
                    shift(&mut board, &mut score, d, bx, bx);
                }
                add_random(&mut board, &mut score, &mut rng, bx, bx);
                render(&board, score);
            }
            _ => {}
        }
    }

    execute!(stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
    println!();
}
