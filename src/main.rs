extern crate piston_window;
extern crate rand;

use piston_window::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

use std::fs::File;
use std::io::BufReader;
use piston_window::graphics::{clear, rectangle};

#[derive(PartialEq, Copy, Clone)]
enum TetrominoKind { I, J, L, O, S, T, Z }


#[derive(Copy, Clone)]
struct Tetromino {
    kind: TetrominoKind,
    color: [f32; 4], // R, G, B, A
    shape: [[u8; 4]; 4]
}


impl Tetromino
{
    const fn new(kind: TetrominoKind) -> Self
    {
        match kind
        {
            TetrominoKind::I => Tetromino { kind: TetrominoKind::I,
                color: [1.0, 1.0, 1.0, 1.0], // білий
                shape: [[0, 0, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 1, 0]] },

            TetrominoKind::J => Tetromino { kind: TetrominoKind::J,
                color: [0.0, 0.0, 1.0, 1.0], // синій
                shape: [[1, 0, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },

            TetrominoKind::L => Tetromino { kind: TetrominoKind::L,
                color: [0.0, 1.0, 1.0, 1.0], // блакитний
                shape: [[0, 0, 1, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },
            TetrominoKind::S => Tetromino { kind: TetrominoKind::S,
                color: [1.0, 0.0, 1.0, 1.0], // пурпуровий
                shape: [[0, 1, 1, 0],
                        [0, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },

            TetrominoKind::Z => Tetromino { kind: TetrominoKind::Z,
                color: [1.0, 0.0, 0.0, 1.0], // червоний
                shape: [[0, 0, 1, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },

            TetrominoKind::O => Tetromino { kind: TetrominoKind::O,
                color: [0.0, 1.0, 0.0, 1.0], // зелений
                shape: [[0, 0, 0, 0],
                        [0, 0, 0, 0],
                        [0, 1, 1, 0],
                        [0, 1, 1, 0]] },

            TetrominoKind::T => Tetromino { kind: TetrominoKind::T,
                color: [1.0, 1.0, 0.0, 1.0], // жовтий
                shape: [[0, 1, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] }
        }
    }
}

type Well = [[u8; 10]; 24];


struct GameState
{
    game_over: bool,
    fall_counter: u32,
    well: Well,
    ttmo_bag: Vec<Tetromino>,
    curr_ttmo: Tetromino,
    next_ttmo: Tetromino,
    ttmo_row: i32,
    ttmo_col: i32,
    key_map: [bool; 7]
}

fn main()
{
    let mut pause = false;
    let mut window: PistonWindow =
        WindowSettings::new("Rustris", [1280, 720])
            .exit_on_esc(true)
            .transparent(false)
            .build()
            .unwrap();

    window.events.set_ups(30);

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let music_sink = rodio::Sink::try_new(&stream_handle).unwrap();
    music_sink.set_volume(0.1);


    let mut blink_counter = 0;

    let mut starter_bag = create_random_bag();
    let starter_first_ttmo = starter_bag.pop().unwrap();
    let starter_second_ttmo = starter_bag.pop().unwrap();

    let mut game_state = GameState {
        game_over: false,
        fall_counter: 0,
        well: [[0u8; 10]; 24],
        ttmo_bag: starter_bag,
        curr_ttmo: starter_first_ttmo,
        next_ttmo: starter_second_ttmo,
        ttmo_row: 2,
        ttmo_col: 3,
        key_map: [false; 7]
    };

    while let Some(event) = window.next()
    {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            if key == Key::R {
                pause = !pause;
            }
        }

        if !pause {
            match event
            {
                Event::Loop(Loop::Render(_args_not_used)) => {
                    render(&mut window, &event,
                           &game_state.ttmo_row, &game_state.ttmo_col, &game_state.curr_ttmo,
                           &game_state.next_ttmo, &mut game_state.well);
                }

                Event::Loop(Loop::Update(_args_not_used)) =>
                    {
                        if game_state.game_over
                        {
                            if blink_counter == 15 {
                                game_state.well = [[0u8; 10]; 24];
                            }
                            if blink_counter == 30 {
                                game_state.well = [[1u8; 10]; 24];
                                blink_counter = 0;
                            }
                            blink_counter += 1;
                        }
                        else {

                            game_update(&mut game_state);

                            if game_state.game_over {
                                music_sink.stop();
                            } else {
                                if music_sink.empty() {
                                    let music_file = File::open("NESTetrisMusic3.ogg").unwrap();
                                    let music_source = rodio::Decoder::new(BufReader::new(music_file)).unwrap();
                                    music_sink.append(music_source);
                                    music_sink.play();
                                }
                            }
                        }
                    }

                Event::Input(Input::Button(button_args), _time_stamp) =>
                    {
                        if button_args.state == ButtonState::Press {
                            track_keys(&mut game_state.key_map, button_args);
                        }
                    }

                _ => {
                    ()
                }
            }
        }
    }
}

fn track_keys(key_map: &mut [bool; 7], btn_info: ButtonArgs)
{
    match btn_info.button
    {
        Button::Keyboard(Key::Left) => key_map[0] = true,
        Button::Keyboard(Key::Right) => key_map[1] = true,
        Button::Keyboard(Key::Up) => key_map[2] = true,
        Button::Keyboard(Key::D) => key_map[2] = true,
        Button::Keyboard(Key::F) => key_map[3] = true,
        Button::Keyboard(Key::Down) => key_map[4] = true,
        Button::Keyboard(Key::Space) => key_map[5] = true,
        _ => ()
    }
}

fn game_update(game_state: &mut GameState)
{
    if game_state.fall_counter < 20 {
        game_state.fall_counter += 1;
    }
    else
    {
        game_state.fall_counter = 0;

        if would_collide(&game_state.curr_ttmo, &game_state.well, &(game_state.ttmo_row + 1), &game_state.ttmo_col)
        {
            freeze_to_well(&game_state.curr_ttmo, &mut game_state.well, &game_state.ttmo_row, &game_state.ttmo_col);
            game_state.well = clear_complete_rows(game_state.well);
            if game_state.ttmo_bag.is_empty() {game_state.ttmo_bag = create_random_bag();}
            game_state.curr_ttmo = game_state.next_ttmo;
            game_state.next_ttmo = game_state.ttmo_bag.pop().unwrap();

            game_state.ttmo_row = 2;
            game_state.ttmo_col = 3;

            if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col)
            {
                game_state.game_over = true;
            }
        }

        else {game_state.ttmo_row += 1;}
    }

    if game_state.key_map[0] && !would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &(game_state.ttmo_col - 1))
    {game_state.ttmo_col -= 1;}

    if game_state.key_map[1] && !would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &(game_state.ttmo_col + 1))
    {game_state.ttmo_col += 1;}

    if game_state.key_map[4] && !would_collide(&game_state.curr_ttmo, &game_state.well, &(game_state.ttmo_row + 1), &game_state.ttmo_col)
    {game_state.ttmo_row += 1;}

    if game_state.key_map[5]
    {
        for row in game_state.ttmo_row..24 {
            if would_collide(&game_state.curr_ttmo, &game_state.well, &row, &game_state.ttmo_col) {
                game_state.ttmo_row = row - 1;
                break;
            }
        }
    }

    if game_state.key_map[2] {
        rotate_tetrimino(&mut game_state.curr_ttmo, false);
        if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col) {
            rotate_tetrimino(&mut game_state.curr_ttmo, true);
        }
    }

    if game_state.key_map[3] {
        rotate_tetrimino(&mut game_state.curr_ttmo, true);
        if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col) {
            rotate_tetrimino(&mut game_state.curr_ttmo, false);
        }
    }

    if game_state.key_map[6] {
        // &mut game_state.curr_ttmo, true
        if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col) {
            rotate_tetrimino(&mut game_state.curr_ttmo, false);
        }
    }

    game_state.key_map = [false; 7];
}

fn create_random_bag() -> Vec<Tetromino>
{
    let mut tetrimino_bag: Vec<Tetromino> = vec![ Tetromino::new(TetrominoKind::I),
                                                  Tetromino::new(TetrominoKind::J),
                                                  Tetromino::new(TetrominoKind::L),
                                                  Tetromino::new(TetrominoKind::O),
                                                  Tetromino::new(TetrominoKind::S),
                                                  Tetromino::new(TetrominoKind::T),
                                                  Tetromino::new(TetrominoKind::Z) ];

    tetrimino_bag.shuffle(&mut thread_rng());
    tetrimino_bag.shuffle(&mut thread_rng());
    tetrimino_bag.shuffle(&mut thread_rng());

    tetrimino_bag
}

fn rotate_tetrimino(ttmo: &mut Tetromino, clockwise: bool)
{
    if ttmo.kind == TetrominoKind::O {return;}

    let source = ttmo.shape;
    let mut rotated: [[u8; 4]; 4] = [[0; 4]; 4];

    let matrix_size: usize;
    if ttmo.kind == TetrominoKind::I {matrix_size = 4;} else {matrix_size = 3;}

    for row in 0..matrix_size
    {
        if clockwise {
            for col in 0..matrix_size {
                rotated[col][(matrix_size - 1) - row] = source[row][col];
            }
        }

        else {
            for col in 0..matrix_size {
                rotated[(matrix_size - 1) - col][row] = source[row][col];
            }
        }
    }

    ttmo.shape = rotated;
}

fn would_collide(ttmo: &Tetromino, well: &Well, row: &i32, col: &i32) -> bool
{
    let mut well_row: i32;
    let mut well_col: i32;

    for ttmo_row in 0..4 {
        for ttmo_col in 0..4 {
            if ttmo.shape[ttmo_row][ttmo_col] == 0 {continue;}
            well_row = ttmo_row as i32 + *row;
            well_col = ttmo_col as i32 + *col;

            if well_col < 0 {return true;}
            if well_col > 9 {return true;}
            if well_row > 23 {return true;}

            if well[well_row as usize][well_col as usize] != 0 {return true;}
        }
    }

    false
}

fn freeze_to_well(ttmo: &Tetromino, well: &mut Well, well_row: &i32, well_col: &i32)
{
    for row in 0..4 {
        for col in 0..4 {
            if ttmo.shape[row][col] == 0 { continue; }
            // println!("well[{}][{}] = 1", (*well_row + row as i32) as usize, (*well_col + col as i32) as usize);
            well[(*well_row + row as i32) as usize][(*well_col + col as i32) as usize] = ttmo.shape[row][col];
        }
    }
}

fn clear_complete_rows(well: Well) -> Well
{
    let mut new_well: Well = [[0; 10]; 24];
    let mut new_well_row: usize = 23;

    for old_well_row in (0..24).rev()
    {
        let mut pop_count = 0;
        for col in 0..10 {
            if well[old_well_row][col] != 0 {pop_count += 1;}
        }

        if pop_count == 0 || pop_count == 10 {continue;}

        if well[old_well_row].iter().sum::<u8>() > 0
        {
            new_well[new_well_row] = well[old_well_row];
            new_well_row -= 1;
        }
    }

    new_well
}

fn render(win: &mut PistonWindow, re: &Event, row: &i32, col: &i32, curr: &Tetromino, next: &Tetromino, well: &Well)
{
    win.draw_2d(re, |_context, graphics, _device| { clear([0.5; 4], graphics);});

    win.draw_2d(re, |context, graphics, _device| { rectangle([0.0, 0.0, 0.0, 1.0], [463.0, -140.0, 354.0, 842.0], context.transform, graphics); });

    draw_well_blocks(win, re, well);
    draw_tetromino_well(win, re, row, col, curr);
    draw_tetromino_pixel(win, re, 320.0, 115.0, next);
}

fn draw_tetromino_well(win: &mut PistonWindow, re: &Event, well_row: &i32, well_col: &i32, ttmo: &Tetromino)
{
    let (x, y) = well_to_pixel(*well_row, *well_col);
    draw_tetromino_pixel(win, re, x, y, ttmo);
}

fn draw_tetromino_pixel(win: &mut PistonWindow, e: &Event, px: f64, py: f64, ttmo: &Tetromino)
{
    for ttmo_row in 0..4 {
        for ttmo_col in 0..4 {
            if ttmo.shape[ttmo_row][ttmo_col] == 0 {continue;}

            let x_offs = px + 35.0 * ttmo_col as f64;
            let y_offs = py + 35.0 * ttmo_row as f64;

            win.draw_2d(e,
            |context, graphics, _device| {
                rectangle(ttmo.color, [x_offs + 1.0, y_offs + 1.0, 33.0, 33.0], context.transform, graphics);
            });
        }
    }
}

fn draw_well_blocks(win: &mut PistonWindow, e: &Event, well: &Well)
{
    for row in 0..24 {
        for col in 0..10 {

            if well[row][col] == 0 { continue; }    // No square to be drawn here.

            let (x_offs, y_offs) = well_to_pixel(row as i32, col as i32);
            win.draw_2d(e,
                        |context, graphics, _device| {
                            // Draw 33x33 square inside 35x35 space.
                            rectangle( [1.0, 1.0, 1.0, 1.0], [x_offs + 1.0, y_offs + 1.0, 33.0, 33.0], context.transform, graphics);
                        }
            );
        }
    }
}

fn well_to_pixel(row: i32, col: i32) -> (f64, f64)
{
    ( (col as f64) * 35.0 + 465.0, (row as f64) * 35.0 - 140.0 )
}