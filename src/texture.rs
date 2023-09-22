/**
 * texture.rs: TGA parser and utility functions.
 */

use std::{fs::{self, File}, io::Read};

use tinytga::{RawTga, RawPixel};
use nalgebra::{Vector2, Vector4};

#[derive(Debug)]
pub struct Texture {
  width: f32,
  height: f32,
  pixels: Vec<RawPixel>,
  pub loaded: bool,
}

impl Texture {
  pub fn new() -> Texture {
    Texture {
      width: 0.0,
      height: 0.0,
      pixels: vec![],
      loaded: false,
    }
  }

  pub fn load(&mut self, filename: &String) {
    let mut f = File::open(filename).expect("File does not exist");
    let metadata = fs::metadata(filename).expect("Unable to read metadata");
    let mut buf = vec![0; metadata.len() as usize];
    f.read_exact(&mut buf).expect("Buffer overflow");

    let tga: RawTga = RawTga::from_slice(&buf[..]).unwrap();
    self.width = tga.header().width as f32;
    self.height = tga.header().height as f32;
    self.pixels = tga.pixels().collect();
    self.loaded = true;
  }
  pub fn get(&self, vec: Vector2<f32>) -> Vector4<f32> {
    let x = (vec.x * self.width).floor();
    let y = (vec.y * self.height).floor();

    match self.pixels.get(((y * self.width) + x) as usize) {
      Some(x) => {
        let r = (x.color >> 16) & 0xff;
        let g = (x.color >> 8) & 0xff;
        let b = x.color & 0xff;
        Vector4::new(
          r as f32,
          g as f32,
          b as f32,
          1.0
        )
      }
      None => Vector4::new(0.0, 0.0, 0.0, 0.0)
    }
  }
}