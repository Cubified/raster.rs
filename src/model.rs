/**
 * model.rs: OBJ parser and associated utility functions.
 */

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::{vec, path::Path};

use nalgebra::{Vector2, Vector3, Vector4};

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
  let file = File::open(filename)?;
  Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
pub struct Model {
  vertex_buffer: Vec<Vector3<f32>>,
  normal_buffer: Vec<Vector3<f32>>,
  uv_buffer: Vec<Vector2<f32>>,

  face_vert: Vec<i32>,
  face_uv: Vec<i32>,
  face_normal: Vec<i32>,

  bbox: [f32; 6],

  pub center: Vector3<f32>,
}

impl Model {
  pub fn new() -> Model {
    Model {
      vertex_buffer: vec![],
      normal_buffer: vec![],
      uv_buffer: vec![],

      face_vert: vec!(),
      face_uv: vec!(),
      face_normal: vec!(),

      bbox: [
        std::f32::MAX, std::f32::MIN,
        std::f32::MAX, std::f32::MIN,
        std::f32::MAX, std::f32::MIN,
      ],

      center: Vector3::zeros(),
    }
  }
  pub fn load_obj(path: &String) -> Result<Model, Box<dyn Error>> {
    let mut out = Model::new();
    let path = Path::new(&path);

    let lines = read_lines(path)?;
    for line in lines {
      let ip = line?;

      let cmd = ip.get(0..2).unwrap_or("");
      if cmd.is_empty() {
        continue;
      }

      match cmd {
        "f " => {
          let mut dests = vec![
            &mut out.face_vert,
            &mut out.face_uv,
            &mut out.face_normal,
          ];
          let sizes = [
            out.vertex_buffer.len(),
            out.uv_buffer.len(),
            out.normal_buffer.len(),
          ];
          for part in ip.split(' ').skip(1) {
            if part.is_empty() {
              continue;
            }

            let pieces: Vec<&str> = part.split('/').collect();
            for i in 0..3 {
              let val = pieces.get(i).unwrap_or(&"0");
              if val.is_empty() {
                continue;
              }

              let dest = &mut dests[i];
              let mut num = val.parse::<i32>().unwrap();
              if num < 0 {
                num += (sizes[i] + 1) as i32;
              }
              dest.push(num);
            }
          }
        },
        "v " | "vn" | "vt" => {
          let mut tmp = vec![];
          for val in ip.split(' ').skip(1) {
            if val.is_empty() {
              continue;
            }
            tmp.push(val.parse::<f32>().unwrap());
          }

          if cmd == "v " {
            out.bbox[0] = out.bbox[0].min(tmp[0]);
            out.bbox[1] = out.bbox[1].max(tmp[0]);


            out.bbox[2] = out.bbox[2].min(tmp[1]);
            out.bbox[3] = out.bbox[3].max(tmp[1]);

            out.bbox[4] = out.bbox[4].min(tmp[2]);
            out.bbox[5] = out.bbox[5].max(tmp[2]);

            out.vertex_buffer.push(Vector3::from_vec(tmp));
          } else if cmd == "vn" {
            out.normal_buffer.push(Vector3::from_vec(tmp).normalize());
          } else if cmd == "vt" {
            out.uv_buffer.push(Vector2::new(tmp[0], 1.0 - tmp[1]));
          }
        },
        "# " => (),
        x => {
          eprintln!("Unrecognized OBJ command: {}", x);
          continue;
        }
      };
    }

    // Approximate center of mass for centering model view matrix.
    //   Works well for symmetric models, much less so for asymmetric ones.
    out.center = Vector3::new(
      (out.bbox[0] + out.bbox[1]) / 2.0,
      (out.bbox[2] + out.bbox[3]) / 2.0,
      (out.bbox[4] + out.bbox[5]) / 2.0,
    );

    Ok(out)
  }

  pub fn nfaces(&self) -> usize {
    self.face_vert.len() / 3
  }

  pub fn uv(&self, iface: usize, nthvert: usize) -> Option<Vector2<f32>> {
    let idx = (iface * 3) + nthvert;
    let opt = self.face_uv.get(idx);
    match opt {
      None | Some(0) => None,
      Some(x) => Some(self.uv_buffer[(*x as usize) - 1]),
    }
  }

  pub fn normal(&self, iface: usize, nthvert: usize) -> Option<Vector3<f32>> {
    let idx = (iface * 3) + nthvert;
    let opt = self.face_normal.get(idx);
    match opt {
      None | Some(0) => None,
      Some(x) => Some(self.normal_buffer[(*x as usize) - 1]),
    }
  }

  pub fn vert(&self, iface: usize, nthvert: usize) -> Vector4<f32> {
    let idx = (iface * 3) + nthvert;
    let face = (*self.face_vert.get(idx).unwrap() as usize) - 1;
    let v = self.vertex_buffer[face];

    Vector4::new(v.x, v.y, v.z, 1.0)
  }
}
