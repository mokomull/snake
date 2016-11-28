mod board;
extern crate termios;

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

fn set_direction(game: &mut board::Game) {
	use std::io::Read;
	use board::Direction::*;

	let mut stdin = std::io::stdin();
	let mut buf = [0; 3];

	match stdin.read(&mut buf) {
		Ok(3) => {
			match &buf {
				// For some reason, these are [ESC] O A (etc.) in zsh, but Rust
				// is receiving [ESC] [ A (etc.).
				b"\x1b[A" => game.set_direction(Up),
				b"\x1b[B" => game.set_direction(Down),
				b"\x1b[C" => game.set_direction(Right),
				b"\x1b[D" => game.set_direction(Left),
				_ => println!("Not an arrow key: {:?}", buf),
			}
		},
		Ok(_) => println!("Got too few bytes."),
		Err(e) => panic!("{}", e),
	}
}

fn main() {
	use termios::{Termios, ICANON, ECHO, TCSANOW, tcsetattr};

	let mut game = board::Game::new(WIDTH, HEIGHT);

	let mut termios = Termios::from_fd(0).unwrap();
	termios.c_lflag &= !(ICANON | ECHO);
	match tcsetattr(0, TCSANOW, &termios) {
		Ok(_) => (),
		Err(x) => panic!("tcsetattr: {}", x),
	}

	loop {
		dump(&game);
		set_direction(&mut game);
		game.tick();
	}
}

#[test]
fn make_sure_match_ref_equality_works_like_i_think() {
	let mut buf = [0 as u8; 3];
	let other = b"\x1bOA";

	buf[0] = b'\x1b';
	buf[1] = b'O';
	buf[2] = b'A';

	assert_eq!(&buf, other);

	match &buf {
		b"\x1bOA" => (),
		_ => panic!("nope"),
	}
}
