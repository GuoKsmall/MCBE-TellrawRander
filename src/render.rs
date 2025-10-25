use std::collections::HashMap;
use image::{Rgba, RgbaImage};
use crate::font::RgbaMatrix;
use crate::define::{ITALIC_CHAR_HORIZON_PADDING, CHAR_HORIZON_PADDING};
use crate::font::{Font, RuneFont};

pub const FMT_Obfuscated: u32 = 1 << 8;
pub const FMT_Bold: u32 = 1 << 9;
pub const FMT_Italic: u32 = 1 << 10;

#[derive(Debug, Clone)]
pub struct SimulateOptions {
    pub font_horizon_padding: i32,
    pub line_padding: i32,
    pub color_mapping: HashMap<String, (u8, u8, u8, u8)>,
}

impl Default for SimulateOptions {
    fn default() -> Self {
        let mut color_mapping = HashMap::new();
        color_mapping.insert("0".to_string(), (0, 0, 0, 255));
        color_mapping.insert("1".to_string(), (0, 0, 170, 255));
        color_mapping.insert("2".to_string(), (0, 170, 0, 255));
        color_mapping.insert("3".to_string(), (0, 170, 170, 255));
        color_mapping.insert("4".to_string(), (170, 0, 0, 255));
        color_mapping.insert("5".to_string(), (170, 0, 170, 255));
        color_mapping.insert("6".to_string(), (255, 170, 0, 255));
        color_mapping.insert("7".to_string(), (170, 170, 170, 255));
        color_mapping.insert("8".to_string(), (85, 85, 85, 255));
        color_mapping.insert("9".to_string(), (85, 85, 255, 255));
        color_mapping.insert("a".to_string(), (85, 255, 85, 255));
        color_mapping.insert("b".to_string(), (85, 255, 255, 255));
        color_mapping.insert("c".to_string(), (255, 85, 85, 255));
        color_mapping.insert("d".to_string(), (255, 85, 255, 255));
        color_mapping.insert("e".to_string(), (255, 255, 85, 255));
        color_mapping.insert("f".to_string(), (255, 255, 255, 255));
        color_mapping.insert("g".to_string(), (221, 214, 5, 255));
        color_mapping.insert("h".to_string(), (222, 214, 5, 255));
        color_mapping.insert("i".to_string(), (227, 212, 209, 255));
        color_mapping.insert("j".to_string(), (68, 58, 59, 255));
        color_mapping.insert("m".to_string(), (151, 22, 7, 255));
        color_mapping.insert("n".to_string(), (180, 104, 77, 255));
        color_mapping.insert("p".to_string(), (222, 177, 45, 255));
        color_mapping.insert("q".to_string(), (17, 160, 54, 255));
        color_mapping.insert("s".to_string(), (44, 186, 168, 255));
        color_mapping.insert("t".to_string(), (33, 73, 123, 255));
        color_mapping.insert("u".to_string(), (154, 92, 198, 255));
        color_mapping.insert("v".to_string(), (235, 114, 20, 255));

        Self {
            font_horizon_padding: CHAR_HORIZON_PADDING,
            line_padding: 6,
            color_mapping,
        }
    }
}

pub struct TellRawSimulator {
    font: RuneFont,
    options: SimulateOptions,
}

impl TellRawSimulator {
    pub fn new(font: RuneFont, options: SimulateOptions) -> Self {
        Self { font, options }
    }

    // Draw font onto canvas
    fn draw(
        &self,
        canvas: &mut RgbaMatrix,
        patch: &Font,
        pos: (i32, i32),
        color: (u8, u8, u8, u8),
    ) {
        let h = patch.height() as i32;
        let w = patch.width() as i32;
        let (start_x, start_y) = pos;
        
        if !patch.colored {
            for y in 0..h {
                for x in 0..w {
                    if start_y + y < canvas.height() as i32 && start_x + x < canvas.width() as i32 {
                        let pixel = patch.mat.get_pixel(x as u32, y as u32);
                        let alpha = pixel[3] as f32 / 255.0;
                        let new_pixel = Rgba([
                            (color.0 as f32 * alpha) as u8,
                            (color.1 as f32 * alpha) as u8,
                            (color.2 as f32 * alpha) as u8,
                            (color.3 as f32 * alpha) as u8,
                        ]);
                        canvas.put_pixel((start_x + x) as u32, (start_y + y) as u32, new_pixel);
                    }
                }
            }
        } else {
            for y in 0..h {
                for x in 0..w {
                    if start_y + y < canvas.height() as i32 && start_x + x < canvas.width() as i32 {
                        let pixel = patch.mat.get_pixel(x as u32, y as u32);
                        canvas.put_pixel((start_x + x) as u32, (start_y + y) as u32, *pixel);
                    }
                }
            }
        }
    }

    // Get color by format
    fn get_color(&self, fmt: u32) -> (u8, u8, u8, u8) {
        let mut color = (255, 255, 255, 255);
        if fmt != 0 {
            let color_code = char::from_u32(fmt & 0x7F).unwrap_or('0');
            if fmt & 0x7F != 0 {
                if let Some(&c) = self.options.color_mapping.get(&color_code.to_string()) {
                    color = c;
                }
            }
        }
        color
    }

    // Split format and text
    fn split_format_and_text(&self, mix: &str) -> (Vec<Vec<String>>, Vec<Vec<u32>>) {
        let mut current_fmt = 0;
        let lines: Vec<&str> = mix.split('\n').collect();
        let mut out_text = Vec::new();
        let mut out_fmt = Vec::new();
        
        for line in lines {
            let mut text = Vec::new();
            let mut fmt = Vec::new();
            let mut is_fmt = false;
            
            for ch in line.chars() {
                if ch == '§' {
                    if is_fmt {
                        text.push(ch.to_string());
                        fmt.push(current_fmt);
                        is_fmt = false;
                    } else {
                        is_fmt = true;
                    }
                } else if is_fmt {
                    match ch {
                        'r' => current_fmt = 0,
                        'l' => current_fmt |= FMT_Bold,
                        'o' => current_fmt |= FMT_Italic,
                        'k' => current_fmt |= FMT_Obfuscated,
                        '0'..='9' | 'a'..='u' => {
                            current_fmt = (current_fmt & 0xFF80) | (ch as u32);
                        }
                        _ => {
                            text.push(ch.to_string());
                            fmt.push(current_fmt);
                        }
                    }
                    is_fmt = false;
                } else {
                    text.push(ch.to_string());
                    fmt.push(current_fmt);
                }
            }
            out_text.push(text);
            out_fmt.push(fmt);
        }
        (out_text, out_fmt)
    }

    fn get_line_width(&mut self, line: &[String], fmt: &[u32]) -> i32 {
        let mut total_width = 0;
        let mut last_fmt = 0;
        
        for (w, &f) in line.iter().zip(fmt.iter()) {
            let font = self.font.get_font(w, f & 0xFF80);
            total_width += font.width() as i32;
            last_fmt = f;
        }
        
        if last_fmt & FMT_Italic != 0 {
            total_width += ITALIC_CHAR_HORIZON_PADDING;
        }
        
        total_width + (line.len() as i32 - 1).max(0) * self.options.font_horizon_padding
    }

    // Shear image for italic effect
    fn shear_image(&self, img: &RgbaMatrix, k: f64) -> RgbaMatrix {
        let h = img.height() as i32;
        let w = img.width() as i32;
        let new_w = (w as f64 + k.abs() * h as f64).ceil() as i32;
        let offset = new_w - w;
        let mut out = RgbaImage::new(new_w as u32, h as u32);
        
        for y in 0..h {
            for x_new in 0..new_w {
                let x = x_new as f64 + k * y as f64 - offset as f64;
                let x_round = x.round() as i32;
                if x_round >= 0 && x_round < w && y >= 0 && y < h {
                    let pixel = img.get_pixel(x_round as u32, y as u32);
                    out.put_pixel(x_new as u32, y as u32, *pixel);
                }
            }
        }
        out
    }

    fn italic(&self, mat: &RgbaMatrix) -> RgbaMatrix {
        let k = 15.0_f64.to_radians().tanh();
        self.shear_image(mat, k)
    }

    pub fn render(&mut self, text: &str) -> RgbaImage {
        let (lines, fmts) = self.split_format_and_text(text);
        let max_width = lines.iter()
            .zip(fmts.iter())
            .map(|(line, fmt)| self.get_line_width(line, fmt))
            .max()
            .unwrap_or(0) as u32;
        
        let height = (lines.len() as u32 * 31 + (lines.len() as u32 - 1).max(0) * self.options.line_padding as u32) as u32;
        let mut mat = RgbaImage::new(max_width, height);
        
        for (line_i, (line, fmt)) in lines.iter().zip(fmts.iter()).enumerate() {
            let start_y = (line_i as u32 * (31 + self.options.line_padding as u32)) as i32;
            let mut start_x = 0;
            let mut italic_start_x = -1;
            
            for (i, (c, &f)) in line.iter().zip(fmt.iter()).enumerate() {
                let pos = (start_x, start_y);
                let patch = self.font.get_font(c, f & 0xFF80);
                self.draw(&mut mat, &patch, pos, self.get_color(f));
                
                if f & FMT_Italic != 0 && italic_start_x == -1 {
                    italic_start_x = start_x;
                }
                
                start_x += patch.width() as i32 + self.options.font_horizon_padding;
                
                if italic_start_x == -1 {
                    continue;
                }
                
                if i != line.len() - 1 && fmt[i + 1] & FMT_Italic != 0 {
                    continue;
                }
                
                let italic_end_x = start_x - self.options.font_horizon_padding;
                let italic_region = image::imageops::crop_imm(
                    &mat,
                    italic_start_x.max(0) as u32,
                    start_y as u32,
                    (italic_end_x - italic_start_x.max(0)) as u32,
                    31,
                ).to_image();
                
                let italic_mat = self.italic(&italic_region);
                let w = italic_mat.width();
                let paste_x = (italic_start_x - 4).max(0) as u32;
                
                for y in 0..31 {
                    for x in 0..w {
                        if paste_x + x < mat.width() && start_y + y < mat.height() as i32 {
                            let pixel = italic_mat.get_pixel(x, y as u32);
                            mat.put_pixel(paste_x + x, (start_y + y) as u32, *pixel);
                        }
                    }
                }
                
                italic_start_x = -1;
            }
        }
        
        mat
    }
}

pub fn render(img_dir_path: &str, text: &str, options: Option<SimulateOptions>) -> RgbaImage {
    let mut simulator = TellRawSimulator::new(
        RuneFont::new(img_dir_path),
        options.unwrap_or_default(),
    );
    simulator.render(text)
}