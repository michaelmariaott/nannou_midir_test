use nannou::prelude::*;
use midir::{MidiOutput, MidiOutputConnection, MidiIO};
use std::error::Error;
use std::io::{stdout, stdin, Write};
use std::{sync::mpsc::Sender, thread};
use std::time::Duration;

const NOTE_ON_MSG: u8 = 0x90;
const _NOTE_OFF_MSG: u8 = 0x80;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    ball: Ball,
    midi_sender: Sender<Note>,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    
    // midi setup
    let (event_tx, event_rx) = std::sync::mpsc::channel::<Note>();
    let event_txx = event_tx.clone();
    thread::spawn(move ||{
        let mut conn_out = connect_midi().unwrap();
        for received in event_rx {
            println!("Received: {:?}", received);
            let midi_message = [NOTE_ON_MSG, received.pitch, received.vel];
            conn_out.send(&midi_message).unwrap();
            if received.vel > 0 {
                let event_txx = event_txx.clone();
                thread::spawn( move || {
                    thread::sleep(Duration::from_millis(received.dur));
                    event_txx.send(Note::new(received.pitch, 0, 0)).unwrap();
                });
            }
        };
    });


    event_tx.send(Note::new(60, 100, 500)).unwrap();
    event_tx.send(Note::new(64, 100, 500)).unwrap();
    event_tx.send(Note::new(67, 100, 500)).unwrap();
    event_tx.send(Note::new(71, 100, 500)).unwrap();

    // ball setup
    let ball = Ball{
        pos: vec2(0.0, 0.0),
        vel: vec2(random_range(1.0, 10.0), random_range(1.0, 10.0)),
        rad: random_range(10.0, 50.0),
    };

    Model { _window, ball, midi_sender: event_tx }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let boundaries = app.window_rect();
    let midi_sender = &model.midi_sender;
    model.ball.update(boundaries, midi_sender);


}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(PLUM);
    model.ball.display(&draw);
    draw.to_frame(app, &frame).unwrap();
}

fn connect_midi() -> Result<MidiOutputConnection, Box<dyn Error>> {
    let midi_out = MidiOutput::new("midir output")?;
    let out_port = select_port(&midi_out)?;
    let conn_out = midi_out.connect(&out_port, "midir-forward")?;
    Ok(conn_out)
}

fn select_port<T: MidiIO>(midi_io: &T) -> Result<T::Port, Box<dyn Error>> {
    let out_ports = midi_io.ports();
    let port = match out_ports.len() {
        0 => return Err("no output port found".into()),
        1 => {
            println!("Choosing the only available output port: {}", midi_io.port_name(&out_ports[0]).unwrap());
            &out_ports[0]
        },
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_io.port_name(p).unwrap());
            }
            print!("Please select output port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            out_ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid output port selected")?
        }
    };
    Ok(port.clone())
}

#[derive(Debug, Clone, Copy)]
struct Note {
    pitch: u8,
    vel: u8,
    dur: u64,
}

impl Note {
    fn new(pitch: u8, vel: u8, dur: u64) -> Note {
        Note {
            pitch,
            vel,
            dur,
        }
    }
}

struct Ball {
    pos: Vec2,
    vel: Vec2,
    rad: f32,
}

impl Ball {
    fn update(&mut self, boundaries: Rect, midi_sender: &Sender<Note>) {
        self.pos += self.vel;

        if self.pos.x + self.rad > boundaries.right() {
            self.vel.x *= -1.0;
            let message = Note::new(67, 100, random_range(300, 2000));
            midi_sender.send(message).unwrap();
        }
        if self.pos.x - self.rad < boundaries.left() {
            self.vel.x *= -1.0;
            let message = Note::new(71, 100, random_range(300, 2000));
            midi_sender.send(message).unwrap();
        }
        if self.pos.y + self.rad > boundaries.top(){
            self.vel.y *= -1.0;
            let message = Note::new(60, 100, random_range(300, 2000));
            midi_sender.send(message).unwrap();
        }
        if self.pos.y - self.rad < boundaries.bottom() {
            self.vel.y *= -1.0;
            let message = Note::new(64, 100, random_range(300, 2000));
            midi_sender.send(message).unwrap();
        }
    }

    fn display(& self, draw: &Draw) {
        draw.ellipse()
            .color(STEELBLUE)
            .x_y(self.pos.x,self.pos.y)
            .radius(self.rad);
    }
}