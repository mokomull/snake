mod board;

use std::time::Duration;
use std::thread::sleep;

const WIDTH: usize = 78;
const HEIGHT: usize  = 20;

fn dump(game: &board::Game) {
	#[cfg(feature = "clear")]
	{
		// Clear screen
		print!("\x1bc");
	}

	print!("┌");
	for _ in 0..WIDTH {
		print!("─");
	}
	print!("┐\n");

	use board::Cell::*;

	for row in 0..HEIGHT {
		print!("│");
		for col in 0..WIDTH {
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
	for _ in 0..WIDTH {
		print!("─");
	}
	print!("┘\n");
}

fn main() {
	let mut game = board::Game::new(WIDTH, HEIGHT);
	let frame_time = Duration::from_millis(500);

	loop {
		dump(&game);
		sleep(frame_time);
		game.tick();
	}
}
