mod board;
extern crate x11_client;

use std::os::unix::net::UnixStream;
use std::io::prelude::*;

use x11_client::*;

const WIDTH: usize = 64;
const HEIGHT: usize  = 64;
const SNAKE_SIZE: usize = 16;

fn dump<T: Write>(stream: &mut T, game: &board::Game,
        window: u32, snake_gc: u32, target_gc: u32, bg_gc: u32) {
	for row in 0..HEIGHT {
		for col in 0..WIDTH {
            let gc = match game.at(col, row).unwrap() {
                board::Cell::Empty => bg_gc,
                board::Cell::Target => target_gc,
                board::Cell::Snake(_) => snake_gc,
            };
            stream.write(&PolyFillRectangle::new(
                window,
                gc,
                (col * SNAKE_SIZE) as i16,
                (row * SNAKE_SIZE) as i16,
                SNAKE_SIZE as u16, SNAKE_SIZE as u16
            ).as_bytes());
		}
	}
}

fn main() {
	let mut game = board::Game::new(WIDTH, HEIGHT);

	let interval = std::time::Duration::from_millis(100);

    let mut socket = UnixStream::connect("/tmp/.X11-unix/X0").unwrap();
    let client_init: Vec<_> = ClientInit::new().into();
    socket.write(&client_init).unwrap();

    let server_init = ServerInit::from_stream(&mut socket).unwrap();
    let window = server_init.resource_id_base + 1;
    let snake_gc = window + 1;
    let bg_gc = snake_gc + 1;
    let target_gc = bg_gc + 1;

    socket.write(&CreateWindow::new(
        24,
        window,
        server_init.roots[0].root,
        0, 0,
        (WIDTH * SNAKE_SIZE) as u16, (HEIGHT * SNAKE_SIZE) as u16,
        0, // border-width
        1, // InputOutput
        0, // visual: CopyFromParent
    ).as_bytes()).unwrap();

    socket.write(&MapWindow::new(window).as_bytes()).unwrap();
    socket.write(&ChangeWmName::new(
        window,
        "Snake".into()
    ).as_bytes()).unwrap();

    socket.write(&CreateGc::new(
        snake_gc, window, 0x00AA00,
    ).as_bytes()).unwrap();

    socket.write(&CreateGc::new(
        bg_gc, window, 0xFFFFFF,
    ).as_bytes()).unwrap();

    socket.write(&CreateGc::new(
        target_gc, window, 0xee9922,
    ).as_bytes()).unwrap();

    socket.set_read_timeout(Some(interval)).unwrap();
    loop {
        let mut buf = [0 as u8; 32];
        let result = socket.read_exact(&mut buf);
        match result {
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                game.tick();
                dump(
                    &mut socket,
                    &game,
                    window, snake_gc, target_gc, bg_gc
                );
                continue;
            }
            Err(e) => { // probably EOF
                println!("Unexpected error: {:?}", e);
                return;
            }
            _ => {}
        }

        let event = Event::from_bytes(&buf);

        match event {
            Event::Expose {..} => dump(
                &mut socket, &game,
                window, snake_gc, target_gc, bg_gc
            ),
            Event::KeyPress { detail: key, .. } => {
                use board::Direction::*;
                match key {
                    111 => game.set_direction(Up),
                    113 => game.set_direction(Left),
                    114 => game.set_direction(Right),
                    116 => game.set_direction(Down),
                    _ => (),
                }
            }
            _ => {
                println!("Unhandled event: {:?}", event);
            }
        }
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
