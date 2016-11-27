// TODO: make Direction private - the UI won't need to know the snake's
// direction.  I saw something about hiding enum contents at one point, but I
// forgot where.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Direction {
	Up,
	Left,
	Down,
	Right,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Cell {
	Empty,
	Target,
	Snake(Direction),
}

pub struct Board {
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

		Board {
			board: board,
		}
	}

	pub fn at(&self, column: usize, row: usize) -> Option<Cell> {
		// .clone() because for some reason it won't copy the referent, even
		// though it's trivial.
		self.board.get(row).and_then(|r| r.get(column)).and_then(|c| Some(c.clone()))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn i_have_no_idea_what_im_doing() {
		let board = Board::new(80, 24);

		assert_eq!(Some(Cell::Empty), board.at(12, 12));
		assert_eq!(None, board.at(80, 0));
		assert_eq!(None, board.at(0, 24));
	}
}
