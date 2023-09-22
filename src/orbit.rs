/**
 * orbit.rs: 3D, mouse-based orbit controls.
 */

use nalgebra::{Vector2, Vector3, Matrix4};

use crate::shader::Shader;

const DAMP: f32 = 0.1;
const FUDGE: f32 = 0.001;

pub struct Orbit<'a> {
  shader: &'a mut Shader<'a>,

  offset: Vector3<f32>,
  spherical: Vector2<f32>,
  sph_delta: Vector2<f32>,
  dist_delta: f32,
  pub distance: f32,
  pub rotate_start: Vector2<f32>,
}

impl Orbit<'_> {
  pub fn new<'a>(s: &'a mut Shader<'a>) -> Orbit<'a> {
    Orbit {
      shader: s,

      offset: Vector3::zeros(),
      spherical: Vector2::zeros(),
      sph_delta: Vector2::new(FUDGE, 0.0),
      dist_delta: 0.0,
      distance: 20.0,
      rotate_start: Vector2::zeros(),
    }
  }

  pub fn mouse_move(&mut self, x: f32, y: f32) {
    let rotate_end = Vector2::new(x, y);
    let delta = rotate_end - self.rotate_start;

    self.sph_delta -= Vector2::new(
      (2.0 * std::f32::consts::PI * delta.x) / (self.shader.height as f32),
      (2.0 * std::f32::consts::PI * delta.y) / (self.shader.height as f32)
    );

    self.rotate_start = rotate_end;
  }

  pub fn zoom(&mut self, amt: f32) {
    self.dist_delta = amt;
  }
  
  pub fn update(&mut self) {
    if self.sph_delta.metric_distance(&Vector2::zeros()) < FUDGE && self.dist_delta.abs() < FUDGE {
      return;
    }

    if (self.dist_delta < FUDGE && self.distance > 1.0) || (self.dist_delta > -FUDGE && self.distance < 1000.0) {
      self.distance += self.dist_delta;
      self.dist_delta *= 1.0 - DAMP;
    } else {
      self.dist_delta = 0.0;
    }

    self.spherical.x = f32::atan2(self.offset.x, self.offset.z);
    self.spherical.y = self.offset.y.acos();
    self.spherical += self.sph_delta * DAMP;
    self.spherical.y = self.spherical.y.clamp(FUDGE, std::f32::consts::PI - FUDGE);
    self.sph_delta *= 1.0 - DAMP;

    let sin_phi = self.spherical.y.sin();
    self.offset.x = sin_phi * self.spherical.x.sin();
    self.offset.y = self.spherical.y.cos();
    self.offset.z = sin_phi * self.spherical.x.cos();

    let matrix = Matrix4::new_translation(&self.offset);
    self.shader.set_modelview(&matrix);
    self.shader.look_at(&(self.offset * self.distance));
    self.shader.render();
  }
}