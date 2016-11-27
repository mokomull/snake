mod board;

fn main() {
    let width = 78;
	let height = 20;

	let board = board::Board::new(width, height);

	#[cfg(feature = "clear")]
	{
		// Clear screen
		print!("\x1bc");
	}

	print!("┌");
	for _ in 0..width {
		print!("─");
	}
	print!("┐\n");

	use board::Cell::*;

	for row in 0..height {
		print!("│");
		for col in 0..width {
			let c = match board.at(col, row).unwrap() {
				Empty => " ",
				Snake(_) => "\u{2588}",
				Target => "\u{2592}",
			};
			print!("{}", c);
		}
		print!("│\n");
	}

	print!("└");
	for _ in 0..width {
		print!("─");
	}
	print!("┘\n");
}
