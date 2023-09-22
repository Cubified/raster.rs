/**
 * esc.rs: Escape sequence parser and state machine.
 */

#[derive(Debug)]
enum States {
  Esc,    // \x1b
  Square, // \x1b[
  Args,   // \x1b[<0;32;64M
}

pub struct EscSeq {
  state: States,
  current_arg: u32,

  pub is_mouse: bool,
  pub args: Vec<u32>,
  pub command: char,
}

impl EscSeq {
  pub fn new() -> EscSeq {
    EscSeq {
      state: States::Esc,
      is_mouse: false,
      args: vec![],
      current_arg: 0,
      command: ' ',
    }
  }
  pub fn parse_one(&mut self, b: char) -> bool {
    match self.state {
      States::Esc => {
        if b == '\x1b' {
          self.state = States::Square;
        }
      },
      States::Square => {
        if b == '[' {
          self.state = States::Args;
        }
      },
      States::Args => {
        match b {
          '<' => { self.is_mouse = true; },
          ';' => {
            self.args.push(self.current_arg);
            self.current_arg = 0;
          }
          '0'..='9' => {
            self.current_arg *= 10;
            self.current_arg += ((b as u8) - b'0') as u32;
          },
          _ => {
            self.args.push(self.current_arg);
            self.current_arg = 0;

            self.command = b;
            return true;
          }
        }
      },
    }
    false
  }
  pub fn reset(&mut self) {
    self.state = States::Esc;
    self.is_mouse = false;
    self.args.clear();
    self.current_arg = 0;
    self.command = ' ';
  }
}