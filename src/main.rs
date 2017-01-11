mod board;
extern crate byteorder;
extern crate x11_client;
extern crate futures;
extern crate tokio_core;
extern crate tokio_uds;

use std::io::prelude::*;
use std::io::Cursor;

use x11_client::*;
use byteorder::{ByteOrder, BigEndian};
use futures::{Future, BoxFuture};
use tokio_uds::UnixStream;
use tokio_core::io::{read_exact, write_all};

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

	let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let socket = UnixStream::connect("/tmp/.X11-unix/X0", &handle).unwrap();
    let client_init: Vec<_> = ClientInit::new().into();
    let f = write_all(socket, &client_init).and_then(|(socket, _)| {
        let server_init_prefix = vec![0 as u8; 8];

        // TODO: the destructuring of the server response length really belongs
        // in x11_client, since it depends on the byteorder that it serialized
        // into ClientInit.
        read_exact(socket, server_init_prefix).and_then(|(socket, mut server_init_prefix)| {
            assert_eq!(1, server_init_prefix[0]);
            let length = BigEndian::read_u16(&server_init_prefix[6..8]);
            read_exact(socket, vec![0 as u8; (length * 4) as usize]).and_then(move |(socket, mut server_init_rest)| {
                let mut server_init_data = Vec::new();
                server_init_data.append(&mut server_init_prefix);
                server_init_data.append(&mut server_init_rest);

                let server_init = ServerInit::from_stream(&mut Cursor::new(server_init_data)).unwrap();
                futures::future::ok((socket, server_init))
            })
        }).and_then(|(socket, server_init)| {
            let window = server_init.resource_id_base + 1;
            let snake_gc = window + 1;
            let bg_gc = snake_gc + 1;
            let target_gc = bg_gc + 1;

            write_all(socket, CreateWindow::new(
                24,
                window,
                server_init.roots[0].root,
                0, 0,
                (WIDTH * SNAKE_SIZE) as u16, (HEIGHT * SNAKE_SIZE) as u16,
                0, // border-width
                1, // InputOutput
                0, // visual: CopyFromParent
            ).as_bytes()).and_then(move |(socket, _)| {
                write_all(socket, MapWindow::new(window).as_bytes())
            }).and_then(move |(socket, _)| {
                write_all(socket, ChangeWmName::new(
                    window,
                    "Snake".into()
                ).as_bytes())
            }).and_then(move |(socket, _)| {
                write_all(socket, CreateGc::new(
                    snake_gc, window, 0x00AA00,
                ).as_bytes())
            }).and_then(move |(socket, _)| {
                write_all(socket, CreateGc::new(
                    bg_gc, window, 0xFFFFFF,
                ).as_bytes())
            }).and_then(move |(socket, _)| {
                write_all(socket, CreateGc::new(
                    target_gc, window, 0xee9922,
                ).as_bytes())
            }).and_then(move |(socket, _)| {
                fn event_handler<T: Read+Write+Send+'static>(socket: T, mut game: board::Game, window: u32, snake_gc: u32, target_gc: u32, bg_gc: u32)
                        -> BoxFuture<(), std::io::Error> {
                    read_exact(socket, [0 as u8; 32]).and_then(move |(mut socket, result)| {
                        let event = Event::from_bytes(&result);

                        // TODO: run this on a timer
                        game.tick();

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
                        event_handler(socket, game, window, snake_gc, target_gc, bg_gc)
                    }).boxed()
                }
                event_handler(socket, game, window, snake_gc, target_gc, bg_gc)
            })
        })
    });

    core.run(f).unwrap();
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
