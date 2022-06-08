extern crate colored;
use colored::*;
use console::{Key, Term};
use rand::seq::SliceRandom;
use std::cmp;
use std::io;

#[derive(PartialEq, Debug)]
enum GameState {
    Init,
    Game,
    Lose,
    Win,
}
#[derive(Clone, PartialEq, Debug)]
enum CellState {
    Closed,
    Open,
    Flag,
}

impl Default for CellState {
    fn default() -> Self {
        CellState::Closed
    }
}

#[derive(Clone, Default, Debug)]
struct Cell {
    mine: bool,
    state: CellState,
    neighbors: usize,
}

#[derive(Debug)]
struct Game {
    state: GameState,
    field: Vec<Cell>,
    field_size: usize,
    mine_count: usize,
}

impl Game {
    fn new(field_size: usize, mine_count: usize) -> Self {
        let cell = Cell {
            ..Default::default()
        };
        Self {
            state: GameState::Init,
            field: vec![cell; field_size * field_size],
            field_size,
            mine_count,
        }
    }

    fn gen(&mut self, cursor: &Cursor) {
        let field_size = self.field_size;
        let mine_count = self.mine_count;

        let mut indices = (0..field_size * field_size).collect::<Vec<usize>>();
        let (xmin, xmax) = (
            if cursor.x == 0 { 0 } else { cursor.x - 1 },
            cmp::min(field_size, cursor.x + 2),
        );
        let (ymin, ymax) = (
            if cursor.y == 0 { 0 } else { cursor.y - 1 },
            cmp::min(field_size, cursor.y + 2),
        );
        //println!("{xmin} {xmax} {ymin} {ymax}");
        for x in xmin..xmax {
            for y in ymin..ymax {
                //println!("{x}:{y}");
                let index = indices
                    .iter()
                    .position(|p| *p == y * field_size + x)
                    .unwrap();
                indices.remove(index);
            }
        }

        //println!("{indices:?}");
        if mine_count > indices.len() {
            self.mine_count = indices.len()
        }

        let mines_indices = indices
            .choose_multiple(&mut rand::thread_rng(), mine_count)
            .collect::<Vec<_>>();

        for i in 0..mines_indices.len() as usize {
            let index = mines_indices[i];
            self.field[*index].mine = true;
        }
        println!("{mines_indices:?}");

        //println!("{self:?}");
        for x in 0..field_size {
            for y in 0..field_size {
                let (xmin, xmax) = (if x == 0 { 0 } else { x - 1 }, cmp::min(field_size, x + 2));
                let (ymin, ymax) = (if y == 0 { 0 } else { y - 1 }, cmp::min(field_size, y + 2));

                self.field[y * field_size + x].neighbors = (xmin..xmax)
                    .map(|x| {
                        (ymin..ymax)
                            .filter(|y| self.field[y * field_size + x].mine)
                            .count()
                    })
                    .sum();
            }
        }

        //io::stdin().read_line(&mut String::new());
    }

    fn draw(&self, cursor: &Cursor) {
        let field_size = self.field_size;
        let mine_count = self.mine_count;

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char); //position the cursor at row 1, column 1:
        for y in 0..field_size {
            for x in 0..field_size {
                let symbol :ColoredString = {
                    let cell = &self.field[y * field_size + x];
                    match cell.state {
                        CellState::Closed => {
                            if cell.mine {
                                if self.state == GameState::Lose {
                                    "*".red()
                                }
                                else if self.state == GameState::Win {
                                    "*".red().on_white()
                                } else {
                                    CLOSED.white().clear()
                                }
                            } else {
                                CLOSED.white().clear()
                            }
                        }
                        CellState::Open => {
                            if cell.mine {
                                "*".black().on_red()
                            } else if cell.neighbors == 0 {
                                " ".clear()
                            } else {
                                cell.neighbors.to_string().white().clear()
                            }
                        }
                        CellState::Flag => {
                            "F".on_red()
                        }
                    }
                };
                print!("{} ", if cursor.x == x && cursor.y == y {symbol.on_yellow()} else {symbol});
            }
            println!();
        }
        println!(
            "{}",
            mine_count
                - self
                    .field
                    .iter()
                    .filter(|c| c.state == CellState::Flag)
                    .count()
        );
        //println!("{:?}", &self.field);
    }

    fn open(&mut self, cursor: &Cursor, by_user: bool) {
        let field_size = self.field_size;
        let field = &mut self.field;
        let current = &mut field[cursor.y * field_size + cursor.x];

        if current.mine {
            self.state = GameState::Lose;
            return;
        }
        if current.state == CellState::Closed {
            current.state = CellState::Open;
        }

        let (xmin, xmax) = (
            if cursor.x == 0 { 0 } else { cursor.x - 1 },
            cmp::min(field_size, cursor.x + 2),
        );
        let (ymin, ymax) = (
            if cursor.y == 0 { 0 } else { cursor.y - 1 },
            cmp::min(field_size, cursor.y + 2),
        );
        let mut flags = 0;
        for x in xmin..xmax {
            for y in ymin..ymax {
                //println!("{x}:{y}");
                if x == cursor.x && y == cursor.y {
                    continue;
                }
                if field[y * field_size + x].state == CellState::Flag {
                    flags += 1;
                }
            }
        }

        let current = &mut field[cursor.y * field_size + cursor.x];

        let mut queue: Vec<(usize, usize)> = Vec::new();
        if (current.neighbors == flags && by_user) || current.neighbors == 0 {
            for x in xmin..xmax {
                for y in ymin..ymax {
                    if x == cursor.x && y == cursor.y
                        || field[y * field_size + x].state != CellState::Closed
                    {
                        continue;
                    } else {
                        queue.push((x, y));
                    }
                }
            }
        }
        for (x, y) in queue {
            self.open(&Cursor { x, y }, false);
        }

        if self
            .field
            .iter()
            .filter(|c| !c.mine && c.state == CellState::Closed)
            .count()
            == 0
        {
            self.state = GameState::Win
        }
    }
}

struct Cursor {
    x: usize,
    y: usize,
}

const CLOSED: &str = "-";

fn main() {
    let field_size = read_int("Please enter game field size.");
    println!("Game field size is {field_size}");
    let mine_count = read_int("Please enter mines count.");
    println!("Mines count is {mine_count}");

    let mut game = Game::new(field_size, mine_count);

    let mut cursor: Cursor = Cursor { x: 0, y: 0 };

    let stdout = Term::buffered_stdout();
    'game_loop: loop {
        game.draw(&cursor);
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
                    if game.state == GameState::Init {
                        game.gen(&cursor);
                        game.state = GameState::Game;
                    }
                    game.open(&cursor, true);
                    if game.state == GameState::Lose || game.state == GameState::Win {
                        break 'game_loop;
                    }
                }
                Key::Char('f') => {
                    game.field[cursor.y * field_size + cursor.x].state =
                        match game.field[cursor.y * field_size + cursor.x].state {
                            CellState::Closed => CellState::Flag,
                            CellState::Flag => CellState::Closed,
                            CellState::Open => continue,
                        };
                }
                _ => (),
            }
        }
    }
    game.draw(&cursor);
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
