#![allow(non_snake_case)] //only for crate name

const SWAP: [usize; 4] = [ //could generate with i-i%2+1-i%2 but eeeghh probably not worth it
    1,
    0,
    3,
    2
];

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
];

extern crate termion;

use std::{
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

use rand::{Rng,SeedableRng,rngs::StdRng};

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

fn print_board(board: &Vec<Vec<i16>>, stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>, length: &i16, apple_pos: (usize, usize), grace: i8) {

    let mut buffer = String::new();

    for row_index in 0..board.len() {
        buffer += Goto(0, 1+row_index as u16).to_string().as_str();

        for col_index in 0..board[row_index].len() {
            
            if (col_index+row_index)%2 == 0 {
                buffer += termion::color::Bg(termion::color::LightBlack).to_string().as_str();
            } else {
                buffer += termion::color::Bg(termion::color::Black).to_string().as_str();
            }
            let mut char_to_print = match apple_pos == (col_index, row_index) {
                true => {buffer += termion::color::Fg(termion::color::LightRed).to_string().as_str(); ''},
                false => ' '
            };
            
            let cell = board[row_index][col_index];
            if cell > -1 {
                let origin: usize = {
                    let mut o = SWAP[(cell.clone()%4) as usize];

                    let candidates: [i16; 4] = [
                        board[row_index.max(1)-1][col_index],
                        board[row_index.min(board.len()-2)+1][col_index],
                        board[row_index][col_index.max(1)-1],
                        board[row_index][col_index.min(board[row_index].len()-2)+1],
                    ];
                    for i in 0..4 {
                        if (candidates[i]+4)%4 == SWAP[i] as i16 && (candidates[i]+4)/4 == cell/4 {
                            o = i;
                            break;
                        }
                    }
                    o
                };

                let segment = match *length - cell / 4  {
                    0 => 0,
                    _ => match cell / 4 != 0 {
                        true => 1,
                        false => 2
                    }
                };
                char_to_print = CHAR_REFERENCE[(cell%4) as usize][origin][segment];
            };

            if cell > -1 {
                buffer += match ((*length - cell/4)%2 == 0, grace != 3) {
                    (true, false) => termion::color::Fg(termion::color::LightGreen).to_string(),
                    (false, false) => termion::color::Fg(termion::color::LightYellow).to_string(),
                    (true, true) => termion::color::Fg(termion::color::Green).to_string(),
                    (false, true) => termion::color::Fg(termion::color::Yellow).to_string()
                }.as_str();
            }
            buffer.push(char_to_print);
            buffer += termion::color::Fg(termion::color::Reset).to_string().as_str();
        }
    }

    write!(stdout, "{}", buffer).unwrap();
}

fn loop1(rx: Receiver<char>) {

    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    let mut board: Vec<Vec<i16>> = vec![vec![-5 as i16; termion::terminal_size().unwrap().0 as usize]; termion::terminal_size().unwrap().1 as usize];
    let mut direction: usize = 3;
    let mut new_direction: usize = 3;
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut nx: usize;
    let mut ny: usize;
    let mut length: i16 = 0;
    let mut apple_pos: (usize, usize) = gen_rand(length, &board);// (board.len()/2, board[0].len()/2);
    let mut grace: i8 = 3;

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    loop {
        thread::sleep(std::time::Duration::from_millis(250));

        //after delay, get keys pressed within delay
        new_direction = match match rx.try_recv() {
            Ok(key) => key,
            Err(_) => ' ',
        } {
            'w' => 0,
            's' => 1,
            'a' => 2,
            'd' => 3,
            _ => new_direction,
        };
        if direction/2 != new_direction/2 {
            direction = new_direction;
        };
        
        nx = match direction as i16 {
            2 => x-1,
            3 => x+1,
            _ => x
        };
        ny = match direction as i16 {
            0 => y-1,
            1 => y+1,
            _ => y
        };
        
        //update snake's position
        board[y][x] = direction as i16 + 4 * length;
        if nx >= board[0].len() || ny >= board.len() || nx == usize::MAX || ny == usize::MAX || board[ny][nx] / 4 > 0 {
            if grace > 0 {
                print_board(&board, &mut stdout, &length, apple_pos, grace);
                grace -= 1;
                continue;
            }
            break;
        };

        
        board[ny][nx] = direction as i16 + 4 * length + 4;

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

        print_board(&board, &mut stdout, &length, apple_pos, grace);

        grace = 3;
    }

    write!(stdout, "Game over!").unwrap();
    stdout.flush().unwrap();


}

fn loop2(tx: SyncSender<char>) {
    let stdin = stdin();

    for c in stdin.events() {
        let evt = c.unwrap();
        let _: bool = match evt {
            Event::Key(ke) => match ke {
                Key::Up => {tx.try_send('w').is_err()},
                Key::Down => {tx.try_send('s').is_err()},
                Key::Left => {tx.try_send('a').is_err()},
                Key::Right => {tx.try_send('d').is_err()},
                Key::Char(k) => match k {
                    'q' => break,
                    x => {
                        let thread_tx = tx.clone();
                        thread_tx.try_send(x).is_err()
                    }
                },
                _ => {false}
            },
            _ => {false}
        };
    }
}

fn gen_rand(seed: i16, board: &Vec<Vec<i16>>) -> (usize, usize) {

    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut index = rng.gen_range(0, board.len()*board[0].len() - seed as usize);
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