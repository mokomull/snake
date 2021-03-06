mod board;
mod x11;

use futures::{Future, IntoFuture, Stream};
use tokio::timer::Interval;
use x11_client::*;

const WIDTH: usize = 15;
const HEIGHT: usize = 15;
const SNAKE_SIZE: usize = 16;

fn dump(
    client: x11::X11Client,
    game: &board::Game,
    window: u32,
    snake_gc: u32,
    target_gc: u32,
    bg_gc: u32,
) -> Box<dyn Future<Item = x11::X11Client, Error = std::io::Error> + Send> {
    /* This function should continue to return a Box, because it is only used
    in a context where a Box is necessary -- a closure which can return one
    of multiple Future types depending on the event being processed */
    let cells = (0..HEIGHT)
        .flat_map(|row| {
            (0..WIDTH).map(move |col| {
                let gc = match game.at(col, row).unwrap() {
                    board::Cell::Empty => bg_gc,
                    board::Cell::Target => target_gc,
                    board::Cell::Snake(_) => snake_gc,
                };
                (row, col, gc)
            })
        })
        .collect::<Vec<_>>();
    Box::new(
        futures::stream::iter_ok(cells).fold(client, move |client, (row, col, gc)| {
            client.poly_fill_rectangle(
                window,
                gc,
                (col * SNAKE_SIZE) as i16,
                (row * SNAKE_SIZE) as i16,
                SNAKE_SIZE as u16,
                SNAKE_SIZE as u16,
            )
        }),
    )
}

fn main() {
    let mut game = board::Game::new(WIDTH, HEIGHT);

    let interval = Interval::new_interval(std::time::Duration::from_millis(100));

    let f = x11::connect_unix(0)
        .and_then(|(server_init, client, events)| {
            let window = server_init.resource_id_base + 1;
            let snake_gc = window + 1;
            let bg_gc = snake_gc + 1;
            let target_gc = bg_gc + 1;

            client
                .create_window(
                    24,
                    window,
                    server_init.roots[0].root,
                    0,
                    0,
                    (WIDTH * SNAKE_SIZE) as u16,
                    (HEIGHT * SNAKE_SIZE) as u16,
                    0, // border-width
                    1, // InputOutput
                    0, // visual: CopyFromParent
                )
                .and_then(move |client| client.map_window(window))
                .and_then(move |client| client.change_wm_name(window, "Snake"))
                .and_then(move |client| client.create_gc(snake_gc, window, 0x00_AA_00))
                .and_then(move |client| client.create_gc(bg_gc, window, 0xFF_FF_FF))
                .and_then(move |client| client.create_gc(target_gc, window, 0xee_99_22))
                .and_then(move |client| {
                    enum Tick {
                        X11Event(Event),
                        TimerFired,
                    }

                    let x11 = events.into_stream().map(Tick::X11Event);
                    let timer = interval
                        .map(|_| Tick::TimerFired)
                        .map_err(|e| panic!("timer failed for no good reason: {:?}", e));

                    let read_or_tick = x11.select(timer);

                    read_or_tick.fold(client, move |client, tick| match tick {
                        Tick::X11Event(event) => {
                            println!("x11");
                            match event {
                                Event::Expose { .. } => {
                                    dump(client, &game, window, snake_gc, target_gc, bg_gc)
                                }
                                Event::KeyPress { detail: key, .. } => {
                                    use crate::board::Direction::*;
                                    match key {
                                        111 => game.set_direction(Up),
                                        113 => game.set_direction(Left),
                                        114 => game.set_direction(Right),
                                        116 => game.set_direction(Down),
                                        _ => (),
                                    };
                                    Box::new(Ok(client).into_future())
                                }
                                _ => {
                                    println!("Unhandled event: {:?}", event);
                                    Box::new(Ok(client).into_future())
                                }
                            }
                        }
                        Tick::TimerFired => {
                            println!("timer");
                            game.tick();
                            dump(client, &game, window, snake_gc, target_gc, bg_gc)
                        }
                    })
                })
        })
        .map(|_| ())
        .map_err(|e| panic!("the stream ended unexpectedly: {:?}", e));

    tokio::run(f);
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
