mod board;

fn main() {
    let width = 78;
	let height = 20;

	let game = board::Game::new(width, height);

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
			let c = match game.at(col, row).unwrap() {
				Empty => " ",
				Snake(_) => "\x1b[32m\u{2588}\x1b[0m",
				Target => "\x1b[31m\u{2592}\x1b[m",
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
