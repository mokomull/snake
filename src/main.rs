#![feature(async_await)]

mod board;
mod x11;

use futures::compat::{Compat, Stream01CompatExt};
use futures::stream::StreamExt;
use tokio::timer::Interval;
use x11_client::*;

const WIDTH: usize = 15;
const HEIGHT: usize = 15;
const SNAKE_SIZE: usize = 16;

async fn dump<'a>(
    client: &'a mut x11::X11Client,
    game: &'a board::Game,
    window: u32,
    snake_gc: u32,
    target_gc: u32,
    bg_gc: u32,
) -> std::io::Result<()> {
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

    for &(row, col, gc) in &cells {
        client
            .poly_fill_rectangle(
                window,
                gc,
                (col * SNAKE_SIZE) as i16,
                (row * SNAKE_SIZE) as i16,
                SNAKE_SIZE as u16,
                SNAKE_SIZE as u16,
            )
            .await?;
    }
    Ok(())
}

async fn main_loop() -> std::io::Result<()> {
    let mut game = board::Game::new(WIDTH, HEIGHT);

    let interval = Interval::new_interval(std::time::Duration::from_millis(100));

    let (server_init, mut client, events) = x11::connect_unix(0).await?;

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
        .await?;

    client.map_window(window).await?;
    client.change_wm_name(window, "Snake").await?;
    client.create_gc(snake_gc, window, 0x00_AA_00).await?;
    client.create_gc(bg_gc, window, 0xFF_FF_FF).await?;
    client.create_gc(target_gc, window, 0xee_99_22).await?;

    enum Tick {
        X11Event(Event),
        TimerFired,
    }

    let x11 = events.into_stream().map(Tick::X11Event);
    let timer = interval.compat().map(|x| {
        x.expect("timer failed for no good reason");
        Tick::TimerFired
    });

    let mut read_or_tick = futures::stream::select(x11, timer);

    loop {
        let event = read_or_tick.next().await;
        match event.expect("stream unexpectedly ended") {
            Tick::X11Event(event) => {
                println!("x11");
                match event {
                    Event::Expose { .. } => {
                        dump(&mut client, &game, window, snake_gc, target_gc, bg_gc).await?
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
                    }
                    _ => {
                        println!("Unhandled event: {:?}", event);
                    }
                }
            }
            Tick::TimerFired => {
                println!("timer");
                game.tick();
                dump(&mut client, &game, window, snake_gc, target_gc, bg_gc).await?;
            }
        }
    }
}

fn main() {
    tokio::run(Compat::new(Box::pin(async move {
        match main_loop().await {
            Err(e) => panic!("I/O error: {}", e),
            Ok(()) => Ok(()),
        }
    })));
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
