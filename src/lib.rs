#![allow(unstable_name_collisions)]
#![allow(non_camel_case_types)]
#![no_std]

mod ffi;

extern crate alloc;

use alloc::{
  string::{String, ToString},
  vec::Vec,
};

use image::{imageops::overlay_bounds, GenericImage, GenericImageView, Pixel, Rgb, RgbaImage};
use imageproc::{
  drawing::{draw_cubic_bezier_curve_mut, draw_line_segment_mut},
  noise::gaussian_noise_mut,
  rect::Rect,
};

mod util {
  use alloc::vec::Vec;

  use fontdue::Font;
  use image::{GrayImage, ImageBuffer, Pixel, Rgba, RgbaImage};

  use crate::overlay_unsafe;
  #[allow(non_upper_case_globals)]
  const empty: Rgba<u8> = Rgba([0, 0, 0, 0]);

  #[derive(Debug)]
  pub struct layout_raster {
    pub width: u32,
    pub height: u32,
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
  }

  pub fn coerce_font_size(height: u32, width: u32, chars: u32) -> f32 { core::cmp::min(height, width / chars) as f32 }

  pub unsafe fn layout_rasterize<T: Copy>(l: fontdue::layout::Layout<T>, fonts: &[Font], r: u8, g: u8, b: u8) -> layout_raster {
    let height = l.height() as u32;
    let glyphs = l.glyphs();
    let l_g = &glyphs.last().unwrap_unchecked();
    let width = (&l_g.width + l_g.x as usize) as u32;
    // let height = (l_g.height + l_g.y as usize) as u32;
    let mut opacity = GrayImage::new(width, height);
    glyphs.iter().for_each(|&glyph| {
      let (metrics, bitmap) = fonts.get_unchecked(glyph.font_index).rasterize_config(glyph.key);
      let bitmap = ImageBuffer::from_vec(metrics.width as u32, metrics.height as u32, bitmap).unwrap_unchecked();
      overlay_unsafe(&mut opacity, &bitmap, glyph.x as u32, glyph.y as u32)
    });
    // Rgba::from([r, g, b]).to_Rgbaa();
    let mut cell = RgbaImage::from_pixel(width, height, empty);
    cell.pixels_mut().zip(opacity.pixels().map(|&pixel| pixel.0[0])).for_each(|(pixel, opacity)| {
      if opacity != 0 {
        let s = &mut pixel.channels_mut()[..];
        core::ptr::copy((&[r, g, b, opacity]).as_ptr(), s.as_mut_ptr(), s.len());
      }
    });

    layout_raster { buffer: cell, width, height }
  }
}

mod js {
  extern "C" {
    pub(crate) fn rand_range(min: u32, max: u32) -> u32;

    pub(crate) fn rgb(min: u32, max: u32) -> *mut u8;
  }
}

unsafe fn get_colors(min: u32, max: u32) -> [u8; 3] { core::slice::from_raw_parts(js::rgb(min, max), 3).try_into().unwrap_unchecked() }

// pub static mut random64: rand::Rand64 = rand::Rand64::new(1);
// pub static mut random32: rand::Rand32 = rand::Rand32::new(1);

pub(crate) unsafe fn overlay_unsafe<I, J>(bottom: &mut I, top: &J, x: u32, y: u32)
where
  I: GenericImage,
  J: GenericImageView<Pixel = I::Pixel>, {
  let c = overlay_bounds(bottom.dimensions(), top.dimensions(), x, y); // (range_width, range_height)
  (0..c.1).for_each(|top_y| {
    (0..c.0).for_each(|top_x| {
      let x = top_x + x;
      let y = top_y + y;
      let mut bottom_pixel = bottom.unsafe_get_pixel(x, y);
      bottom_pixel.blend(&top.unsafe_get_pixel(top_x, top_y));
      bottom.unsafe_put_pixel(x, y, bottom_pixel);
    });
  });
}

unsafe fn draw_filled_rect_mut_unsafe<C>(canvas: &mut C, rect: Rect, color: C::Pixel)
where
  C: GenericImage,
  C::Pixel: 'static, {
  let canvas_bounds = Rect::at(0, 0).of_size(canvas.width(), canvas.height());
  if let Some(intersection) = canvas_bounds.intersect(rect) {
    (0..intersection.height()).for_each(|dy| {
      (0..intersection.width()).for_each(|dx| {
        let x = intersection.left() as u32 + dx;
        let y = intersection.top() as u32 + dy;
        canvas.unsafe_put_pixel(x, y, color)
      });
    });
  }
}

pub struct captcha {
  buffer: Vec<u8>,
  solution: String,
  width: u32,
  height: u32,
}

#[no_mangle]
unsafe extern "C" fn font_new(ptr: ffi::mem::buf, len: usize, scale: u16) -> *mut fontdue::Font {
  ffi::ptr::pack(
    fontdue::Font::from_bytes(ffi::io::load(ptr, len), fontdue::FontSettings {
      scale: scale as f32,
      ..fontdue::FontSettings::default()
    })
    .unwrap_unchecked(),
  )
}

#[no_mangle]
pub unsafe extern "C" fn font_free(ptr: *mut fontdue::Font) { ffi::ptr::drop(ptr); }

#[no_mangle]
pub unsafe extern "C" fn captcha_free(ptr: *mut captcha) { ffi::ptr::drop(ptr); }

#[no_mangle]
unsafe extern "C" fn draw_captcha(font: *mut fontdue::Font, width: u32, height: u32, tptr: ffi::mem::buf, tlen: usize) -> *mut captcha {
  let mut image = RgbaImage::new(width, height);
  let captcha_code = alloc::string::String::from_utf8_unchecked(ffi::io::load(tptr, tlen));

  draw_filled_rect_mut_unsafe(&mut image, Rect::at(0, 0).of_size(width, height), Rgb(get_colors(160, 210)).to_rgba());

  let font_size = util::coerce_font_size(height, width, captcha_code.len() as u32);
  let fonts = &[(*font).clone()];
  // let font = (*font).clone();
  captcha_code.chars().enumerate().for_each(|(i, ch)| {
    let colors: [u8; 3] = get_colors(0, 140);
    let colors: [u8; 4] = [0u8, *colors.get_unchecked(0), *colors.get_unchecked(1), *colors.get_unchecked(2)];
    let bubble_px = font_size / 6.0;
    let mut x = i as f32 * font_size + js::rand_range((bubble_px * -1.0) as u32, bubble_px as u32) as f32;
    if x < 0.0 {
      x = 2.0;
    }
    let mut max_y = height as f32 - font_size;
    if max_y <= 1.0 {
      max_y = 1.0;
    }
    let y = js::rand_range(0, max_y as u32) as f32;
    let mut l = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
    l.append(fonts, &fontdue::layout::TextStyle::new(&ch.to_string(), font_size, 0));
    let ras = util::layout_rasterize(l, fonts, *colors.get_unchecked(1), *colors.get_unchecked(2), *colors.get_unchecked(3));
    overlay_unsafe(&mut image, &ras.buffer, x as u32, y as u32);
  });

  (0..js::rand_range(1, 3)).for_each(|_| {
    let deep_line_color = Rgb(get_colors(0, 140)).to_rgba();
    let start_point = (js::rand_range(0, width) as f32, js::rand_range(0, height) as f32);
    let end_point = (js::rand_range(0, width) as f32, js::rand_range(0, height) as f32);
    draw_line_segment_mut(&mut image, start_point, end_point, deep_line_color);
  });

  (0..js::rand_range(1, 2)).for_each(|_| {
    let deep_line_color = Rgb(get_colors(0, 140)).to_rgba();
    let start_point = (js::rand_range(0, width) as f32, js::rand_range(0, height) as f32);
    let end_point = (js::rand_range(0, width) as f32, js::rand_range(0, height) as f32);
    let bezier_point1 = (js::rand_range(0, width) as f32, js::rand_range(0, height) as f32);
    let bezier_point2 = (js::rand_range(0, width) as f32, js::rand_range(0, height) as f32);

    draw_cubic_bezier_curve_mut(&mut image, start_point, end_point, bezier_point1, bezier_point2, deep_line_color);
  });

  gaussian_noise_mut(&mut image, 15.0, 5.0, 10);

  ffi::ptr::pack(captcha {
    buffer: image.into_raw(),
    solution: captcha_code,
    width,
    height,
  })
}

#[no_mangle]
unsafe extern "C" fn captcha_buffer(ptr: *mut captcha) -> ffi::mem::buf { ffi::io::peek(&(*ptr).buffer) }

#[no_mangle]
unsafe extern "C" fn captcha_solution(ptr: *mut captcha) -> ffi::mem::buf { ffi::io::peek(&(*ptr).solution.as_bytes()) }

#[no_mangle]
unsafe extern "C" fn captcha_as_png(ptr: *mut captcha, compression: u8) -> ffi::mem::buf {
  let mut buf = Vec::new();
  {
    let mut enc = png::Encoder::new(&mut buf, (*ptr).width, (*ptr).height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_compression(if compression == 0 { png::Compression::Fast } else { png::Compression::Default });

    enc.set_compression(match compression {
      0 => png::Compression::Default,
      1 => png::Compression::Fast,
      2 => png::Compression::Best,
      _ => png::Compression::Default,
    });

    enc.set_depth(png::BitDepth::Eight);

    let mut writer = match enc.write_header() {
      Ok(writer) => writer,
      Err(_) => return ffi::ptr::err(0),
    };

    if writer.write_image_data(&(*ptr).buffer).is_err() {
      return ffi::ptr::err(1);
    }
  }

  ffi::io::store(buf)
}
