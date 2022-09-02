#![allow(non_snake_case)] //only for crate name

const CHAR_REFERENCE : [[[char; 3]; 4]; 4] = [
    [ //from up
        ['?', '?', '?'], //to up
        ['▲', '║', '|'], //to down
        ['▲', '╝', '┘'], //to left
        ['▲', '╚', '└'] //to right
    ],
    [ //from down
        ['▼', '║', '|'],
        ['?', '?', '?'],
        ['▼', '╗', '┐'],
        ['▼', '╔', '┌']
    ],
    [ //from left
        ['◄', '╝', '┘'],
        ['◄', '╗', '┐'],
        ['?', '?', '?'],
        ['◄', '═', '-']
    ],
    [ //from right
        ['►', '╚', '└'],
        ['►', '╔', '┌'],
        ['►', '═', '-'],
        ['?', '?', '?']
    ]
    //empty
    //[[' ', ' ', ' '], [' ', ' ', ' '], [' ', ' ', ' '], [' ', ' ', ' ']]
];

const HEIGHT: usize = 16;
const WIDTH: usize = 32;

extern crate termion;

use std::{
    io::{stdin, stdout, Write},
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
};
use termion::{
    event::{Event, Key},
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
};

use rand::{Rng,SeedableRng,rngs::StdRng};

fn main() {
    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

    let (tx, rx) = sync_channel(2);

    let bleh = thread::spawn(move || {
        loop2(tx);
    });

    thread::spawn(|| {
        loop1(rx);
    });

    bleh.join().expect("oops! the child thread panicked");

    write!(stdout, "Exited.").unwrap();
    stdout.flush().unwrap();
}

fn print_board(board: &Vec<Vec<i16>>, stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>, length: &i16, apple_pos: (usize, usize)) {
    write!(stdout, "{}", termion::clear::All).unwrap();

    let mut buffer = String::new();

    buffer.push('┌');
    for _ in 0..WIDTH {buffer += "─";};
    buffer.push('┐');
    buffer += "\n\r";

    for row_index in 0..HEIGHT {
        buffer.push('│');
        for col_index in 0..WIDTH {

            let mut char_to_print = match apple_pos == (col_index, row_index) {
                true => '',
                false => ' ',
            };
            
            let cell = board[row_index][col_index];
            if cell >= 4 {
                let mut origin = match (cell.clone()%4) as usize {
                    0 => 1,
                    1 => 0,
                    2 => 3,
                    3 => 2,
                    _ => 4
                };
                //gotta shorten this
                if board[row_index.max(1)-1][col_index] / 4 == cell / 4 - 1 && board[row_index.max(1)-1][col_index].is_positive() {
                    origin = 0;
                }
                else if board[(row_index+1).min(HEIGHT-1)][col_index] / 4 == cell / 4 - 1 && board[(row_index+1).min(HEIGHT-1)][col_index].is_positive() {
                    origin = 1;
                }
                else if board[row_index][col_index.max(1)-1] / 4 == cell / 4 - 1 && board[row_index][col_index.max(1)-1].is_positive() { 
                    origin = 2;
                }
                else if board[row_index][(col_index+1).min(WIDTH-1)] / 4 == cell / 4 - 1 && board[row_index][(col_index+1).min(WIDTH-1)].is_positive() {
                    origin = 3;
                }

                let segment = match cell / 4 -1 {
                    0 => 2,
                    _ => match cell / 4 == *length {
                        true => 0,
                        false => 1
                    }
                };
                char_to_print = CHAR_REFERENCE[(cell%4) as usize][origin][segment];
            };
            buffer.push(char_to_print);
        }
        buffer.push('│');
        buffer.push('\n');
        buffer.push('\r');
    }

    buffer.push('└');
    for _ in 0..WIDTH {buffer += "─";};
    buffer.push('┘');
    buffer += "\n\r";
    write!(stdout, "{}", buffer).unwrap();
}

fn loop1(rx: Receiver<char>) {
    //this can be the game loop?
    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

    let mut board: Vec<Vec<i16>> = vec![vec![0 as i16; WIDTH]; HEIGHT];
    let mut direction: usize = 3;
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut length: i16 = 1;
    let mut apple_pos: (usize, usize) = (WIDTH/2, HEIGHT/2);

    loop {
        thread::sleep(std::time::Duration::from_millis(110));
        
        //remove 4 from every cell
        for row in board.iter_mut() {
            for cell in row.iter_mut() {
                *cell = (*cell - 4).max(0);
            }
        }
        
        let input_key = match rx.try_recv() {
            Ok(id) => id,
            Err(_) => '.',
        };
        direction = match input_key {
            'w' => 0,
            's' => 1,
            'a' => 2,
            'd' => 3,
            _ => direction
        };

        board[y][x] = direction as i16+4*length-4; //optimize

        match direction {
            0 => {
                if y > 0 {
                    y -= 1;
                }
            },
            1 => {
                if y < HEIGHT - 1 {
                    y += 1;
                }
            },
            2 => {
                if x > 0 {
                    x -= 1;
                }
            },
            3 => {
                if x < WIDTH - 1 {
                    x += 1;
                }
            },
            _ => {}
        }

        board[y][x] = direction as i16+4*length; //optimize
        
        
        if apple_pos == (x, y) {
            length += 1;
            apple_pos = gen_rand(length, &board);
            for row in board.iter_mut() {
                for cell in row.iter_mut() {
                    if *cell > 0 {
                        *cell = *cell + 4;
                    }
                }
            }
        }

        print_board(&board, &mut stdout, &length, apple_pos);
    }
}

fn loop2(tx: SyncSender<char>) {
    let stdin = stdin();

    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(ke) => match ke {
                Key::Up => tx.send('w').unwrap(),
                Key::Down => tx.send('s').unwrap(),
                Key::Left => tx.send('a').unwrap(),
                Key::Right => tx.send('d').unwrap(),
                Key::Char(k) => match k {
                    'q' => break,
                    x => {
                        let thread_tx = tx.clone();

                        match thread_tx.try_send(x) {
                            Ok(_) => (),
                            Err(_) => (),
                        };
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
}

fn gen_rand(seed: i16, board: &Vec<Vec<i16>>) -> (usize, usize) {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut index = rng.gen_range(0, WIDTH*HEIGHT - seed as usize);
    for row_index in 0..HEIGHT {
        for col_index in 0..WIDTH {
            if board[row_index][col_index] == 0 {
                index -= 1;
            }
            if index == 0 {
                return (col_index, row_index);
            }
        }
    }
    (0, 0)
}