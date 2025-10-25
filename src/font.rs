use std::collections::HashMap;
use std::path::Path;
use image::{ImageBuffer, Rgba, RgbaImage};
use crate::define::SPACE_WIDTH;

pub type RgbaMatrix = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Debug, Clone)]
pub struct Font {
    pub mat: RgbaMatrix,
    pub colored: bool,
}

impl Font {
    pub fn new(mat: RgbaMatrix, colored: bool) -> Self {
        Self { mat, colored }
    }

    pub fn width(&self) -> u32 {
        self.mat.width()
    }

    pub fn height(&self) -> u32 {
        self.mat.height()
    }

    pub fn clone(&self) -> Self {
        Self {
            mat: self.mat.clone(),
            colored: self.colored,
        }
    }
}

pub trait FontMaker {
    fn get_font(&mut self, rune: &str, fmt: u32) -> Font;
}

impl FontMaker for RuneFont {
    fn get_font(&mut self, rune: &str, fmt: u32) -> Font {
        self.get_font(rune, fmt)
    }
}

pub struct RuneFont {
    root_dir: String,
    cached_group: HashMap<u32, (RgbaImage, bool)>,
    cached_rune: HashMap<(String, u32), Font>,
}

impl RuneFont {
    pub fn new(root_dir: &str) -> Self {
        Self {
            root_dir: root_dir.to_string(),
            cached_group: HashMap::new(),
            cached_rune: HashMap::new(),
        }
    }

    pub fn rune_to_idx(rune: &str) -> (u32, u32, u32) {
        let chars: Vec<char> = rune.chars().collect();
        if chars.is_empty() {
            return (0, 0, 0);
        }
        let code = chars[0] as u32;
        (code >> 8, (code & 0xF0) >> 4, code & 0xF)
    }

    pub fn rune_to_raw_idx(rune: &str) -> u32 {
        let code = rune.encode_utf16().collect::<Vec<u16>>();
        if code.len() >= 1 { code[code.len() - 1] as u32 } else { 0 }
    }

    pub fn idx_to_rune(group: u32, row: u32, col: u32) -> String {
        let idx = group * (16 * 16) + row * 16 + col;
        let code = idx as u16;
        char::from_u32(code as u32).unwrap_or(' ').to_string()
    }

    fn get_group(&mut self, group_idx: u32) -> Option<(RgbaImage, bool)> {
        if let Some(cached) = self.cached_group.get(&group_idx) {
            return Some(cached.clone());
        }

        let file_path = format!("{}/glyph_{:02X}.png", self.root_dir, group_idx);
        if !Path::new(&file_path).exists() {
            // Create empty font image if file not found
            let empty_img = RgbaImage::new(512, 512);
            self.cached_group.insert(group_idx, (empty_img.clone(), false));
            return Some((empty_img, false));
        }

        let img = image::open(&file_path).ok()?;
        let rgba_img = img.to_rgba8();
        let resized = image::imageops::resize(&rgba_img, 512, 512, image::imageops::FilterType::Nearest);
        
        // Check if colored
        let colored = !self.is_grayscale(&resized);
        
        self.cached_group.insert(group_idx, (resized.clone(), colored));
        Some((resized, colored))
    }

    fn is_grayscale(&self, img: &RgbaImage) -> bool {
        for pixel in img.pixels() {
            if pixel[0] != pixel[1] || pixel[1] != pixel[2] {
                return false;
            }
        }
        true
    }

    fn tight_font(&self, square: &RgbaImage) -> RgbaImage {
        let bbox = self.get_bbox(square);
        let (x1, x2) = if let Some((x1, _, x2, _)) = bbox {
            (x1, x2)
        } else {
            (0, SPACE_WIDTH as u32)
        };
        
        image::imageops::crop_imm(square, x1, 0, x2 - x1, 31).to_image()
    }

    fn get_bbox(&self, img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
        let mut min_x = img.width();
        let mut min_y = img.height();
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for (x, y, pixel) in img.enumerate_pixels() {
            if pixel[3] > 0 { // Alpha > 0
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        if found {
            Some((min_x, min_y, max_x + 1, max_y + 1))
        } else {
            None
        }
    }

    pub fn get_font(&mut self, rune: &str, fmt: u32) -> Font {
        let key = (rune.to_string(), fmt);
        if let Some(cached) = self.cached_rune.get(&key) {
            return cached.clone();
        }

        let (g, r, c) = Self::rune_to_idx(rune);
        let page = self.get_group(g);
        
        let font = if let Some((png, colored)) = page {
            let posx = c * 32;
            let posy = r * 32;
            let cropped = image::imageops::crop_imm(&png, posx, posy, 32, 31).to_image();
            let tighted = self.tight_font(&cropped);
            
            let mut font = Font::new(tighted, colored);
            
            if fmt != 0 && !font.colored {
                font = font.clone();
                if fmt & 0x100 != 0 { // FMT_Obfuscated
                    for pixel in font.mat.pixels_mut() {
                        pixel[0] = 1;
                        pixel[1] = 1;
                        pixel[2] = 1;
                    }
                }
                if fmt & 0x200 != 0 { // FMT_Bold
                    let h = font.mat.height();
                    let w = font.mat.width();
                    let pad = 2;
                    let mut new_mat = RgbaImage::new(w + pad, h);
                    
                    for y in 0..h {
                        for x in 0..w {
                            let pixel = font.mat.get_pixel(x, y);
                            for off in 0..pad {
                                if x + off < w + pad {
                                    new_mat.put_pixel(x + off, y, *pixel);
                                }
                            }
                        }
                    }
                    font.mat = new_mat;
                }
            }
            
            font
        } else {
            // Fallback to space
            self.get_font(" ", fmt)
        };

        self.cached_rune.insert(key, font.clone());
        font
    }
}