/**
 * ui.rs: Standard input collector and escape sequence handler.
 */

use std::time::{Duration, SystemTime};
use std::{io, thread};
use std::io::Read;
use std::sync::mpsc::{self, TryRecvError};

use crate::shader::{self, Shader};
use crate::esc::EscSeq;
use crate::orbit::Orbit;

const FPS: u32 = 60;
const FRAME_INTERVAL: Duration = Duration::new(0, (1e9 as u32) / FPS);

pub struct UI<'a> {
  seq: EscSeq,
  orb: Orbit<'a>,
}

impl UI<'_> {
  pub fn new<'a>(s: &'a mut Shader<'a>) -> UI<'a> {
    UI {
      seq: EscSeq::new(),
      orb: Orbit::new(s),
    }
  }
  pub fn handle(&mut self) {
    match self.seq.command {
      'M' => 'M: {
        if !self.seq.is_mouse || self.seq.args.len() < 3 {
          break 'M;
        }

        match self.seq.args[0] {
          // Mouse down
          0 => {
            self.orb.rotate_start.x = self.seq.args[1] as f32;
            self.orb.rotate_start.y = self.seq.args[2] as f32;
          },
          // Mouse move (down)
          32 => {
            self.orb.mouse_move(
              self.seq.args[1] as f32,
              self.seq.args[2] as f32
            );
          },
          // Scroll up
          64 => {
            self.orb.zoom(-1.0);
          }
          // Scroll down
          65 => {
            self.orb.zoom(1.0);
          },
          _ => (),
        }
      },
      _ => { /* Not implemented */ }
    }
    self.seq.reset();
  }
  pub fn run(&mut self) {
    /* Raw mode */
    let mut tio = libc::termios {
      c_iflag: 0,
      c_oflag: 0,
      c_cflag: 0,
      c_lflag: 0,
      c_cc: [0; 20],
      c_ispeed: 0,
      c_ospeed: 0
    };
    unsafe {
      libc::tcgetattr(libc::STDIN_FILENO, &mut tio);

      let mut raw = tio;
      raw.c_lflag &= !(libc::ECHO | libc::ICANON);
      libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw);
    }

    ctrlc::set_handler(move || {
      unsafe {
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &tio);
      }
      shader::stop();
      std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    shader::start();

    let (tx, rx) = mpsc::channel::<(Vec<u8>, usize)>();
    let mut bytes: [u8; 64] = [0; 64];
    thread::spawn(move || loop {
      let n = io::stdin().read(&mut bytes).unwrap();

      tx.send((bytes.to_vec(), n)).unwrap();
    });

    loop {
      let start = SystemTime::now();
      match rx.try_recv() {
          Ok((bytes, n)) => {
            for b in bytes.iter().take(n) {
              if self.seq.parse_one(*b as char) {
                self.handle();
              }
            }
          },
          Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
          _ => (),
      }

      self.orb.update();

      let end = SystemTime::now();
      let dur = end.duration_since(start).unwrap();
      if dur < FRAME_INTERVAL {
        thread::sleep(FRAME_INTERVAL - dur);
      }
    }
  }
}