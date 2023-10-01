/**
 * shader.rs: Vertex and fragment shader routines.
 * 
 * Responsible for printing the output escape sequences to the screen.
 */

use std::collections::HashMap;
use std::io::{self, Write};

use crate::model::Model;
use crate::vertex::Vertex;
use crate::texture::Texture;

use nalgebra::{Vector3, Vector4, Matrix3, Matrix4};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

const ONES: Vector4<f32> = Vector4::new(1.0, 1.0, 1.0, 1.0);

#[derive(Debug)]
pub struct Shader<'a> {
  pub width: i32,
  pub height: i32,

  model: &'a mut Model,

  light: Vector4<f32>,
  uniform_light: Vector3<f32>,

  pub model_view: Matrix4<f32>,
  pub projection: Matrix4<f32>,
  pub viewport: Matrix4<f32>,

  diffuse_map: Texture,
  normal_map: Texture,
  specular_map: Texture,
}

impl Shader<'_> {
  pub fn new(w: usize, h: usize, m: &mut Model) -> Shader {
    Shader {
      width: w as i32,
      height: h as i32,

      model: m,

      light: Vector4::identity(),
      uniform_light: Vector3::identity(),

      model_view: Matrix4::identity(),
      projection: Matrix4::identity(),
      viewport: Matrix4::identity(),

      diffuse_map: Texture::new(),
      normal_map: Texture::new(),
      specular_map: Texture::new(),
    }
  }

  pub fn vertex(&self, vert: &mut Vertex, iface: usize, nthvert: usize) -> Vector4<f32> {
    if let Some(x) = self.model.uv(iface, nthvert) {
      vert.varying_uv.set_column(nthvert, &x);
    }

    if let Some(x) = self.model.normal(iface, nthvert) {
      let prod = self.model_view.try_inverse().unwrap().transpose() * Vector4::new(x.x, x.y, x.z, 0.0);
      vert.varying_normal.set_column(nthvert, &prod.xyz());
    }

    let out = self.model_view * self.model.vert(iface, nthvert);
    vert.view_triangle.set_column(nthvert, &out.xyz());
    self.projection * out
  }

  pub fn fragment(&self, vert: &Vertex, worldspace: Vector3<f32>, screenspace: Vector3<f32>) -> Vector4<f32> {
    let bn = (vert.varying_normal * worldspace).normalize();
    let mut uv = vert.varying_uv * screenspace;
    uv.y = 1.0 - uv.y;

    let ai = match Matrix3::from_columns(&[
      vert.view_triangle.column(1) - vert.view_triangle.column(0),
      vert.view_triangle.column(2) - vert.view_triangle.column(0),
      bn
    ]).try_inverse() {
      Some(ai) => ai,
      None => { return Vector4::zeros(); }
    };
    let i = ai * Vector3::new(vert.varying_uv.m12 - vert.varying_uv.m11, vert.varying_uv.m13 - vert.varying_uv.m11, 0.0);
    let j = ai * Vector3::new(vert.varying_uv.m22 - vert.varying_uv.m21, vert.varying_uv.m23 - vert.varying_uv.m21, 0.0);
    let b = Matrix3::from_columns(&[
      i.normalize(),
      j.normalize(),
      bn
    ]).transpose();

    let normal = (b * (match self.normal_map.loaded {
      true => self.normal_map.get(uv).zyx(),
      false => worldspace,
    })).normalize();
    let specular = match self.specular_map.loaded {
      true => self.specular_map.get(uv).z,
      false => 0.0,
    };

    let factor = match self.diffuse_map.loaded {
        true => 2.0,
        false => 1.0,
    };
    let result_diffuse = normal.dot(&self.uniform_light).max((factor - 1.0) * 0.65);
    let diffuse_contrib = ((normal * result_diffuse * 2.0) - self.uniform_light).normalize();
    let result_specular = (-diffuse_contrib.z).max(0.0).powf(5.0 + specular);

    let diffuse = (match self.diffuse_map.loaded {
      true => self.diffuse_map.get(uv),
      false => ONES * 255.0,
    }).scale(factor * (result_diffuse + result_specular)).add_scalar(10.0);
    Vector4::new(
      diffuse.x.clamp(0.0, 255.0),
      diffuse.y.clamp(0.0, 255.0),
      diffuse.z.clamp(0.0, 255.0),
      1.0
    )
  }

  pub fn set_light(&mut self, x: f32, y: f32, z: f32) {
    self.light = Vector4::new(x, y, z, 0.0).normalize();
    self.uniform_light = (self.model_view * self.light).xyz();
  }

  pub fn set_projection(&mut self, f: f32) {
    self.projection = Matrix4::new(
      1.0, 0.0, 0.0, 0.0,
      0.0, -1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, -1.0 / f, 0.0,
    );
  }

  pub fn look_at(&mut self, eye: &Vector3<f32>) {
    let z = (self.model.center - eye).normalize();
    let x = Vector3::y().cross(&z).normalize();
    let y = z.cross(&x).normalize();

    let m_inv = Matrix4::new(
      x.x, x.y, x.z, 0.0,
      y.x, y.y, y.z, 0.0,
      z.x, z.y, z.z, 0.0,
      0.0, 0.0, 0.0, 1.0,
    );
    let tr = Matrix4::new(
      1.0, 0.0, 0.0, -eye.x,
      0.0, 1.0, 0.0, -eye.y,
      0.0, 0.0, 1.0, -eye.z,
      0.0, 0.0, 0.0, 1.0,
    );

    self.model_view = m_inv * tr;
  }

  pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
    self.viewport = Matrix4::new(
      w / 2.0, 0.0, 0.0, x + w / 2.0,
      0.0, h / 2.0, 0.0, y + h / 2.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    );
  }

  pub fn render(&mut self) {
    let result: Vec<Vec<(i32, f32, String)>> =
      (0..self.model.nfaces()).into_par_iter().map(|i| {
        let mut vert = Vertex::new();
        for j in 0..3 {
          vert.clip[j] = self.vertex(&mut vert, i, j);
        }
        vert.triangle(self)
      }).collect();

    let mut pixels: HashMap<i32, (f32, String)> = HashMap::with_capacity((self.width * self.height) as usize);
    for items in result {
      for (idx, z, color) in items {
        if let Some((prev_z, _)) = pixels.get(&idx) {
          if *prev_z < z {
            continue;
          }
        }

        pixels.insert(idx, (z, color));
      }
    }

    let mut esc = String::from("\x1b[0m\x1b[0H");
    let mut prev = String::new();
    let default = (0.0, String::from("\x1b[0m "));
    for y in 1..self.height {
      for x in 1..self.width {
        let (_, cell) = pixels.get(&(x + (y * self.width))).unwrap_or(&default);

        if *cell == prev {
          esc += " ";
        } else {
          esc += cell;
          prev.clone_from(cell);
        }
      }
      esc += "\n";
    }
    let _ = io::stdout().write_all(esc.as_bytes());
  }

  pub fn set_diffuse(&mut self, filename: &String) {
    self.diffuse_map.load(filename);
  }
  pub fn set_normal(&mut self, filename: &String) {
    self.normal_map.load(filename);
  }
  pub fn set_specular(&mut self, filename: &String) {
    self.specular_map.load(filename);
  }
}

pub fn start() {
  println!("\x1b[?1049h\x1b[0m\x1b[2J\x1b[?1003h\x1b[?1015h\x1b[?1006h\x1b[?25l");
}
pub fn stop() {
  println!("\x1b[0m\x1b[2J\x1b[?1049l\x1b[?1003l\x1b[?1015l\x1b[?1006l\x1b[?25h");
}
