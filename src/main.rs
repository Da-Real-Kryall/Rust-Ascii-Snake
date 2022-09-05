#![allow(non_snake_case)] //only for crate name


/*
delay

get next direction

buffer new location

check if new location is valid
    if not valid and grace is false, make grace true and skip to start of sequence

check if apple is eaten at said location
    if apple is eaten, set apple eaten to true
    increment score by one
    
if apple is eaten, don't remove four from snake segments


*/
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

const HEIGHT: usize = 12;
const WIDTH: usize = 20;

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
            if cell > -1 {
                let origin: usize = {
                    let mut o = SWAP[(cell.clone()%4) as usize];

                    let candidates: [i16; 4] = [
                        board[row_index.max(1)-1][col_index],
                        board[row_index.min(HEIGHT-2)+1][col_index],
                        board[row_index][col_index.max(1)-1],
                        board[row_index][col_index.min(WIDTH-2)+1],
                    ];
                    for i in 0..4 {
                        if (candidates[i]+4)%4 == SWAP[i] as i16 && (candidates[i]+4)/4 == cell/4 {
                            o = i;
                            break;
                        }
                    }
                    o
                };

                let segment = match cell / 4  {
                    0 => 2,
                    _ => match cell / 4 == *length {
                        true => 0,
                        false => 1
                    }
                };
                char_to_print = CHAR_REFERENCE[(cell%4) as usize][origin][segment];
            };
            //buffer.push(' ');
            buffer.push(char_to_print);
            //buffer.push((cell/4+49) as u8 as char); //for debugging, 48 is ascii for 0
            //buffer.push((cell%4+49) as u8 as char); //for debugging, 48 is ascii for 0
        }
        buffer.push('│');
        buffer.push('\n');
        //buffer.push('\n');
        buffer.push('\r');
    }

    buffer.push('└');
    for _ in 0..WIDTH {buffer += "─";};
    buffer.push('┘');
    buffer += "\n\r";
    write!(stdout, "{}", buffer).unwrap();
}

fn loop1(rx: Receiver<char>) {

    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    let mut board: Vec<Vec<i16>> = vec![vec![-5 as i16; WIDTH]; HEIGHT];
    let mut direction: usize = 3;
    let mut new_direction: usize = 3;
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut nx: usize;
    let mut ny: usize;
    let mut length: i16 = 1;
    let mut apple_pos: (usize, usize) = (WIDTH/2, HEIGHT/2);
    let mut grace: i8 = 5;

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
        
        if nx >= WIDTH || ny >= HEIGHT || nx == usize::MAX || ny == usize::MAX || board[ny][nx] / 4 > 0 {
            if grace > 0 {
                grace -= 1;
                continue;
            }
            break;
        };

        //update snake's position
        board[y][x] = direction as i16 + 4 * length;
        
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

        print_board(&board, &mut stdout, &length, apple_pos);

        grace = 5;
    }

    write!(stdout, "Game over!").unwrap();
    stdout.flush().unwrap();


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

                        thread_tx.try_send(x).unwrap();
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