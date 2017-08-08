extern crate rand;

// TODO: make Direction private - the UI won't need to know the snake's
// direction.  I saw something about hiding enum contents at one point, but I
// forgot where.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Direction {
    Up = 1,
    Left = 2,
    Down = 3,
    Right = 4,
}

impl From<usize> for Direction {
    fn from(u: usize) -> Direction {
        match u {
            1 => Direction::Up,
            2 => Direction::Left,
            3 => Direction::Down,
            _ => Direction::Right,
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Cell {
    Empty,
    Target,
    Snake(Direction),
}

struct Board {
    // Indexed by row, then by column, to make printing to the screen easier later.
    board: Vec<Vec<Cell>>,
}

use self::Cell::*;
impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        let mut board = Vec::<Vec<Cell>>::with_capacity(height);

        for _row in 0..height {
            board.push(vec![Empty; width])
        }

        Board { board: board }
    }

    pub fn at(&self, column: usize, row: usize) -> Option<Cell> {
        // .clone() because for some reason it won't copy the referent, even
        // though it's trivial.
        self.board.get(row).and_then(|r| r.get(column)).cloned()
    }
}

pub struct Game {
    board: Board,
    head: (usize, usize),
    tail: (usize, usize),
    width: usize,
    height: usize,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Game {
        let mut board = Board::new(width, height);
        let column = width / 2;
        let row = height / 2;

        // TODO: poor encapsulation of the board
        board.board[0][0] = Target;
        board.board[row][column] = Snake(Direction::Down);

        Game {
            board: board,
            head: (column, row),
            tail: (column, row),
            width: width,
            height: height,
        }
    }

    pub fn get_direction(&self) -> Direction {
        let (column, row) = self.head;
        match self.board.at(column, row) {
            Some(Snake(d)) => d,
            _ => panic!("The snake doesn't have a head"),
        }
    }

    pub fn set_direction(&mut self, d: Direction) {
        let (column, row) = self.head;
        // TODO: poorly encapsulated.
        self.board.board[row][column] = Snake(d);
    }

    pub fn tick(&mut self) {
        let (column, row) = self.head;
        match self.next(self.get_direction(), column, row) {
            Some((new_column, new_row)) => {
                match self.board.at(new_column, new_row) {
                    Some(Empty) => {
                        // Move forward normally - shrink the tail
                        let (column, row) = self.tail;
                        let d = match self.board.at(column, row).unwrap() {
                            Snake(d) => d,
                            _ => panic!("not a snake at the tail"),
                        };
                        // if the tail moves into a wall, the player already died
                        let (new_tcol, new_trow) = self.next(d, column, row).unwrap();

                        // TODO: encapsulation violation, again
                        self.board.board[new_row][new_column] = Snake(self.get_direction());
                        self.board.board[row][column] = Empty;
                        self.head = (new_column, new_row);
                        self.tail = (new_tcol, new_trow);
                    }
                    Some(Target) => {
                        // You ate the target - leave the tail and grow
                        // TODO: board encapsulation.  Yet again.
                        self.board.board[new_row][new_column] = Snake(self.get_direction());
                        self.head = (new_column, new_row);

                        // Randomly generate a new target
                        let mut rng = rand::thread_rng();
                        let mut col_range = rand::distributions::Range::new(0, self.width);
                        let mut row_range = rand::distributions::Range::new(0, self.height);
                        use board::rand::distributions::Sample;
                        loop {
                            let (new_col, new_row) =
                                (col_range.sample(&mut rng), row_range.sample(&mut rng));

                            match self.board.at(new_col, new_row) {
                                Some(Empty) | Some(Target) => {
                                    self.board.board[new_row][new_col] = Target;
                                    break;
                                }
                                _ => (), // keep going
                            }
                        }
                    }
                    Some(Snake(_)) => panic!("You died."),
                    None => panic!("You died."),
                }
            }
            None => panic!("You died."),
        }
    }

    fn next(&self, d: Direction, column: usize, row: usize) -> Option<(usize, usize)> {
        use self::Direction::*;
        match d {
            Left => {
                if column > 0 {
                    Some((column - 1, row))
                } else {
                    None
                }
            }
            Right => self.board.at(column + 1, row).map(|_| (column + 1, row)),
            Up => {
                if row > 0 {
                    Some((column, row - 1))
                } else {
                    None
                }
            }
            Down => self.board.at(column, row + 1).map(|_| (column, row + 1)),
        }
    }

    pub fn at(&self, column: usize, row: usize) -> Option<Cell> {
        self.board.at(column, row)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Cell::*;

    #[test]
    fn i_have_no_idea_what_im_doing() {
        let board = super::Board::new(80, 24);

        assert_eq!(Some(Empty), board.at(12, 12));
        assert_eq!(None, board.at(80, 0));
        assert_eq!(None, board.at(0, 24));
        assert_eq!(Some(Empty), board.at(79, 0));
        assert_eq!(Some(Empty), board.at(0, 23));
    }

    #[test]
    fn game_tick() {
        use super::Direction::*;

        let mut game = Game::new(80, 24);
        game.set_direction(Up);

        let (column, row) = game.head;
        assert_eq!(Some(Snake(Up)), game.board.at(column, row));

        game.tick();
        assert_eq!((column, row - 1), game.head);
        assert_eq!((column, row - 1), game.tail);

        let (column, row) = game.head;
        game.set_direction(Left);
        game.tick();
        assert_eq!((column - 1, row), game.head);
        assert_eq!(game.head, game.tail);

        let (column, row) = game.head;
        game.set_direction(Right);
        game.tick();
        assert_eq!((column + 1, row), game.head);
        assert_eq!(game.head, game.tail);

        let (column, row) = game.head;
        game.set_direction(Down);
        game.tick();
        assert_eq!((column, row + 1), game.head);
        assert_eq!(game.head, game.tail);
    }

    #[test]
    fn target() {
        use super::Direction::*;

        let mut game = Game::new(80, 24);
        game.set_direction(Up);

        for row in 0..24 {
            for col in 0..80 {
                match game.board.at(col, row) {
                    Some(Empty) => game.board.board[row][col] = Target,
                    _ => (),
                }
            }
        }

        let (column, row) = game.head;
        let orig_tail = game.tail;
        game.tick();
        assert_eq!((column, row - 1), game.head);
        assert_eq!(orig_tail, game.tail);
    }
}
