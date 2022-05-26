extern crate colored;
use colored::*;
use console::{Key, Term};
use rand::seq::SliceRandom;
use std::cmp;
use std::io;

//#[derive(Debug)]
struct Cursor {
    x: usize,
    y: usize,
}

const CLOSED: char = '-';

fn main() {
    let field_size = read_int("Please enter game field size.");
    println!("Game field size is {field_size}");
    let mut mines_count = read_int("Please enter mines count.");
    println!("Mines count is {mines_count}");

    let mut field = vec![CLOSED; field_size * field_size];

    println!("{field:?}");

    let mut cursor: Cursor = Cursor { x: 0, y: 0 };

    let mut genered: bool = false;

    let stdout = Term::buffered_stdout();
    'game_loop: loop {
        draw(&field, &field_size, &mines_count, &cursor, true);
        if let Ok(key) = stdout.read_key() {
            println!("{key:?}");
            match key {
                Key::Char('w') => cursor.y -= if cursor.y > 0 { 1 } else { 0 },
                Key::Char('a') => cursor.x -= if cursor.x > 0 { 1 } else { 0 },
                Key::Char('s') => cursor.y += if cursor.y < field_size - 1 { 1 } else { 0 },
                Key::Char('d') => cursor.x += if cursor.x < field_size - 1 { 1 } else { 0 },
                Key::ArrowUp => cursor.y -= if cursor.y > 0 { 1 } else { 0 },
                Key::ArrowLeft => cursor.x -= if cursor.x > 0 { 1 } else { 0 },
                Key::ArrowDown => cursor.y += if cursor.y < field_size - 1 { 1 } else { 0 },
                Key::ArrowRight => cursor.x += if cursor.x < field_size - 1 { 1 } else { 0 },
                Key::Char('q') => break 'game_loop,
                Key::Escape => break 'game_loop,
                Key::Char(' ') => {
                    if !genered {
                        gen_field(&field_size, &mut mines_count, &mut field, &cursor);
                        genered = true;
                    }
                    if !open(&field_size, &mut mines_count, &mut field, &cursor, true) {
                        draw(&field, &field_size, &mines_count, &cursor, false);
                        break 'game_loop;
                    }
                }
                Key::Char('f') => {
                    let current = &mut field[cursor.y * field_size + cursor.x];
                    *current = match *current {
                        CLOSED => {
                            mines_count -= 1;
                            'f'
                        }
                        'f' => {
                            mines_count += 1;
                            CLOSED
                        }
                        '*' => {
                            mines_count -= 1;
                            'F'
                        }
                        'F' => {
                            mines_count += 1;
                            '*'
                        }
                        _ => 'E',
                    }
                }
                _ => (),
            }
        }
    }
}

fn draw(field: &Vec<char>, field_size: &usize, mines_count: &usize, cursor: &Cursor, closed: bool) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); //position the cursor at row 1, column 1:
    for y in 0..*field_size {
        for x in 0..*field_size {
            let char = if x == cursor.x && y == cursor.y {
                'o'
            } else {
                let char = field[y * field_size + x];
                if char == '*' {
                    if closed {
                        CLOSED
                    } else {
                        print!("{} ", "*".red());
                        continue;
                    }
                } else if char == 'f' || char == 'F' {
                    print!("{} ", "F".on_red());
                    continue;
                } else if char == '0' {
                    ' '
                } else {
                    char
                }
            };
            print!("{char} ");
        }
        println!();
    }
    println!("{mines_count}");
}

fn read_int(s: &str) -> usize {
    loop {
        println!("{s}");

        let mut int = String::new();

        io::stdin()
            .read_line(&mut int)
            .expect("Failed to read line");

        if let Ok(num) = int.trim().parse() {
            return num;
        }
    }
}

fn open(
    field_size: &usize,
    mines_count: &mut usize,
    field: &mut Vec<char>,
    cursor: &Cursor,
    by_player: bool,
) -> bool {
    //println!("{:?}", cursor);
    if field[cursor.y * field_size + cursor.x] == '*' {
        return false;
    }
    let (xmin, xmax) = (
        if cursor.x == 0 { 0 } else { cursor.x - 1 },
        cmp::min(*field_size, cursor.x + 2),
    );
    let (ymin, ymax) = (
        if cursor.y == 0 { 0 } else { cursor.y - 1 },
        cmp::min(*field_size, cursor.y + 2),
    );
    let mut mines = 0;
    let mut flags = 0;
    for x in xmin..xmax {
        for y in ymin..ymax {
            //println!("{x}:{y}");
            if x == cursor.x && y == cursor.y {
                continue;
            }

            let current = &field[y * field_size + x];
            match current {
                '*' => mines += 1,
                'F' => {
                    if by_player {
                        flags += 1
                    } else {
                        mines += 1
                    }
                }
                'f' => {
                    if by_player {
                        flags += 1
                    } else {
                        mines += 1
                    }
                }
                _ => (),
            }
        }
    }
    field[cursor.y * field_size + cursor.x] = char::from_digit(mines + flags, 10).unwrap();
    if mines == 0 {
        for x in xmin..xmax {
            for y in ymin..ymax {
                if x == cursor.x && y == cursor.y || field[y * field_size + x] != CLOSED {
                    continue;
                }
                open(field_size, mines_count, field, &Cursor { x, y }, false);
            }
        }
    }
    true
}

fn gen_field(field_size: &usize, mines_count: &mut usize, field: &mut Vec<char>, cursor: &Cursor) {
    let mut indices = (0..field_size * field_size).collect::<Vec<usize>>();
    let (xmin, xmax) = (
        if cursor.x == 0 { 0 } else { cursor.x - 1 },
        cmp::min(*field_size, cursor.x + 2),
    );
    let (ymin, ymax) = (
        if cursor.y == 0 { 0 } else { cursor.y - 1 },
        cmp::min(*field_size, cursor.y + 2),
    );
    //println!("{xmin} {xmax} {ymin} {ymax}");
    for x in xmin..xmax {
        for y in ymin..ymax {
            //println!("{x}:{y}");
            let index = indices
                .iter()
                .position(|p| *p == y * *field_size + x)
                .unwrap();
            indices.remove(index);
        }
    }

    //println!("{indices:?}");
    if *mines_count > indices.len() {
        *mines_count = indices.len()
    }

    let mines_indices = indices
        .choose_multiple(&mut rand::thread_rng(), *mines_count)
        .collect::<Vec<_>>();

    for i in 0..mines_indices.len() as usize {
        let index = mines_indices[i];
        field[*index] = '*';
    }
    //println!("{mines_indices:?}");
}
