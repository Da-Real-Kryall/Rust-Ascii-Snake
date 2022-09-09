#![allow(non_snake_case)] //only for crate name

extern crate termion;

use std::{
    env::consts::OS,
    io::{stdin, stdout, Write},
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
};
use termion::{
    cursor::Goto,
    event::{Event, Key},
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
};

use rand::{rngs::StdRng, Rng, SeedableRng};

const SWAP: [usize; 4] = [
    //could generate with i-i%2+1-i%2 but eeeghh probably not worth it
    1, 0, 3, 2,
];

const COLOUR_PALETTES: [[u8; 2]; 8] = [
    [3, 4], //yellow and blue
    [5, 2], //magenta and green
    [6, 1], //cyan and red
    [1, 2], //red and green
    [2, 3], //green and yellow
    [4, 6], //blue and cyan
    [3, 6], //yellow and cyan
    [1, 4], //red and blue
];

const CHAR_REFERENCE: [[[char; 3]; 4]; 4] = [
    [
        //from up
        ['?', '?', '?'],       //to up
        ['▲', '║', '│'], //to down
        ['▲', '╝', '┘'], //to left
        ['▲', '╚', '└'], //to right
    ],
    [
        //from down
        ['▼', '║', '│'],
        ['?', '?', '?'],
        ['▼', '╗', '┐'],
        ['▼', '╔', '┌'],
    ],
    [
        //from left
        ['◄', '╝', '┘'],
        ['◄', '╗', '┐'],
        ['?', '?', '?'],
        ['◄', '═', '–'],
    ],
    [
        //from right
        ['►', '╚', '└'],
        ['►', '╔', '┌'],
        ['►', '═', '–'],
        ['?', '?', '?'],
    ],
];

fn main() {
    let _stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

    let (tx, rx) = sync_channel(2);

    let bleh = thread::spawn(move || {
        loop2(tx);
    });

    thread::spawn(|| {
        loop1(rx);
    });

    bleh.join().expect("oops! the child thread panicked");
}

fn print_board(
    board: &Vec<Vec<i16>>,
    stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
    length: &i16,
    apple_pos: (usize, usize),
    grace: i8,
    colour_seed: usize,
) {
    let mut print_buffer: String = String::new();
    //let mut rng = StdRng::seed_from_u64((apple_pos.1 * apple_pos.0) as u64);


    for row_index in 0..board.len() {
        for col_index in 0..board[row_index].len() {
            //if the apple or snake is to the left already, don't add a goto
            //else, add a goto to the correct position
            if col_index == 0 || !(board[row_index][col_index.max(1) - 1] != -5
                || apple_pos == (col_index.max(1) - 1, row_index))
            {
                print_buffer.push_str(&format!("{}", Goto(col_index as u16+1, row_index as u16+1)));
            }

            //if the apple is here, write the apple
            let cell = board[row_index][col_index];
            if apple_pos == (col_index, row_index) {
                //random choice between ansi red yellow and green
                print_buffer.push_str(&format!(
                    "\x1b[91m{}",
                    //one in 50 chance seeded with applle_pos.0 * apple_pos.1
                    //if it's 35, use a ඞ instead of an apple
                    if StdRng::seed_from_u64((apple_pos.0 * apple_pos.1) as u64).gen_range(0, 50) == 35 {
                        'ඞ'
                    } else {
                        match OS {
                            "linux" => '@',
                            "macos" => '',
                            _ => 'ඞ',
                        }
                    }
                ));
            }
            //if snake is here, write the snake
            else if cell > -1 {
                let origin: usize = {
                    let mut o = SWAP[(cell.clone() % 4) as usize];

                    let candidates: [i16; 4] = [
                        board[row_index.max(1) - 1][col_index],
                        board[row_index.min(board.len() - 2) + 1][col_index],
                        board[row_index][col_index.max(1) - 1],
                        board[row_index][col_index.min(board[row_index].len() - 2) + 1],
                    ];
                    for i in 0..4 {
                        if (candidates[i] + 4) % 4 == SWAP[i] as i16
                            && (candidates[i] + 4) / 4 == cell / 4
                        {
                            o = i;
                            break;
                        }
                    }
                    o
                };

                let segment = match *length - cell / 4 {
                    0 => 0,
                    _ => match cell / 4 != 0 {
                        true => 1,
                        false => 2,
                    },
                };

                print_buffer.push_str(&format!(
                    "\x1b[{}m{}",
                    COLOUR_PALETTES[colour_seed][(length - cell / 4) as usize % 2]
                        + if grace == 3 { 90 } else { 30 },
                    CHAR_REFERENCE[(cell % 4) as usize][origin][segment]
                ));
            } else if cell > -5 {
                print_buffer.push_str(" ");
            }
        }
    }
    write!(stdout, "{}", print_buffer).unwrap();
}

fn loop1(rx: Receiver<char>) {
    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    let mut board: Vec<Vec<i16>> =
        vec![
            vec![-5 as i16; termion::terminal_size().unwrap().0 as usize];
            termion::terminal_size().unwrap().1 as usize
        ];

    let mut direction: usize = 3;
    let mut new_direction: usize;
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut nx: usize;
    let mut ny: usize;
    let mut length: i16 = 0;
    let mut apple_pos: (usize, usize) = gen_rand(length, &board); // (board.len()/2, board[0].len()/2);
    let mut grace: i8 = 3;

    let seed: usize = rand::thread_rng().gen_range(0, 8);

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    //print spaces until the board is full
    for _ in 0..board.len() {
        for _ in 0..board[0].len() {
            print!(" ");
        }
    }

    loop {
        thread::sleep(std::time::Duration::from_millis(150));
        //after delay, get keys pressed within delay
        new_direction = match match rx.try_recv() {
            Ok(key) => key,
            Err(_) => ' ',
        } {
            'w' => 0,
            's' => 1,
            'a' => 2,
            'd' => 3,
            _ => direction,
        };

        nx = match new_direction as i16 {
            2 => x - 1,
            3 => x + 1,
            _ => x,
        };
        ny = match new_direction as i16 {
            0 => y - 1,
            1 => y + 1,
            _ => y,
        };
        if (nx >= board[0].len()
            || ny >= board.len()
            || nx == usize::MAX
            || ny == usize::MAX
            || board[ny][nx] / 4 > 0)
            && direction != new_direction
        {
            //check if the direction just changed
            if direction != new_direction {
                //if the player did changge the direction, don't punish them and don't change the direction
                //new_direction = direction;
                nx = match direction as i16 {
                    2 => x - 1,
                    3 => x + 1,
                    _ => x,
                };
                ny = match direction as i16 {
                    0 => y - 1,
                    1 => y + 1,
                    _ => y,
                };
                new_direction = direction;
            }
        }

        //if snake is overlapping the border or the snake:
        if nx >= board[0].len()
            || ny >= board.len()
            || nx == usize::MAX
            || ny == usize::MAX
            || board[ny][nx] / 4 > 0
        {
            //check if the direction just changed
            if direction != new_direction {
                //if the player did changge the direction, don't punish them and don't change the direction
                //new_direction = direction;
                nx = match direction as i16 {
                    2 => x - 1,
                    3 => x + 1,
                    _ => x,
                };
                ny = match direction as i16 {
                    0 => y - 1,
                    1 => y + 1,
                    _ => y,
                };
                new_direction = direction;
            } else {
                //if the player didn't change the direction, punish them
                if grace == 0 {
                    break;
                }
                grace -= 1;
                print_board(&board, &mut stdout, &length, apple_pos, grace, seed);
                continue;
            }
        }

        board[ny][nx] = new_direction as i16 + 4 * length + 4;
        board[y][x] = new_direction as i16 + 4 * length;

        if apple_pos == (nx, ny) {
            length += 1;
            apple_pos = gen_rand(length, &board);
        } else {
            for row in board.iter_mut() {
                for cell in row.iter_mut() {
                    *cell = (*cell - 4).max(-5);
                }
            }
        }
        x = nx.clone();
        y = ny.clone();
        direction = new_direction;

        print_board(&board, &mut stdout, &length, apple_pos, grace, seed);

        grace = 3;
    }
    stdout.flush().unwrap();
    panic!("Game over!");
}

fn loop2(tx: SyncSender<char>) {
    let stdin = stdin();

    for c in stdin.events() {
        let evt = c.unwrap();
        let _: bool = match evt {
            Event::Key(ke) => match ke {
                Key::Up => tx.try_send('w').is_err(),
                Key::Down => tx.try_send('s').is_err(),
                Key::Left => tx.try_send('a').is_err(),
                Key::Right => tx.try_send('d').is_err(),
                Key::Char(k) => match k {
                    'q' => break,
                    x => {
                        let thread_tx = tx.clone();
                        thread_tx.try_send(x).is_err()
                    }
                },
                _ => false,
            },
            _ => false,
        };
    }
}

fn gen_rand(seed: i16, board: &Vec<Vec<i16>>) -> (usize, usize) {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut index = rng.gen_range(0, board.len() * board[0].len() - seed as usize);
    for row_index in 0..board.len() {
        for col_index in 0..board[0].len() {
            if board[row_index][col_index] <= -1 {
                index -= 1;
            }
            if index == 0 {
                return (col_index, row_index);
            }
        }
    }
    (0, 0)
}
