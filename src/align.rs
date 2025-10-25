use crate::define::{BOLD_PAD, SPACE_WIDTH, CHAR_HORIZON_PADDING, ITALIC_CHAR_HORIZON_PADDING};
use crate::font_widths::FontWidths;
use crate::utils::find_closest;

// Font widths data
lazy_static::lazy_static! {
    static ref FONT_WIDTHS: Vec<u8> = {
        vec![12; 65536] // Default width for all characters
    };
}

pub fn get_char_width(char: &str, bold: bool) -> i32 {
    if char.is_empty() {
        return 0;
    }
    
    let char_code = char.chars().next().unwrap() as u32;
    let font_widths = FontWidths::new();
    let mut width = font_widths.get_width(char_code) as i32;
    
    if bold {
        width += BOLD_PAD;
    }
    
    width
}

pub fn get_line_width(line: &str) -> i32 {
    if line.contains('\n') {
        panic!("Line contains newline; use get_lines_length instead");
    }
    
    let mut width = 0;
    let mut bold = false;
    let mut _italic = false;
    let mut fmt = false;
    let mut length = 0;
    
    for ch in line.chars() {
        if ch == '§' {
            fmt = true;
            continue;
        } else if fmt {
            fmt = false;
            match ch {
                'l' => bold = true,
                'o' => _italic = true,
                'r' => {
                    bold = false;
                    _italic = false;
                }
                _ => {}
            }
            continue;
        }
        length += 1;
        width += get_char_width(&ch.to_string(), bold);
    }
    
    width += (length - 1).max(0) * CHAR_HORIZON_PADDING;
    if _italic {
        width += ITALIC_CHAR_HORIZON_PADDING;
    }
    width
}

pub fn get_lines_width(lines: &[String]) -> i32 {
    lines.iter().map(|line| get_line_width(line)).max().unwrap_or(0)
}

pub fn get_specific_length_spaces(length: i32) -> String {
    get_specific_length_spaces_and_diff(length, 0).0
}

pub fn get_specific_length_spaces_and_diff(length: i32, prev_diff: i32) -> (String, i32) {
    let (solutions, min_diff) = find_closest(
        SPACE_WIDTH + CHAR_HORIZON_PADDING,
        SPACE_WIDTH + BOLD_PAD + CHAR_HORIZON_PADDING,
        length + prev_diff,
    );
    let (a, b, _) = solutions[0];
    let s = format!("§l{}§r{}", " ".repeat(b as usize), " ".repeat(a as usize));
    (s, min_diff as i32)
}

pub fn cut_by_length(line: &str, spaces: i32) -> Vec<String> {
    let mut width = 0;
    let spaces_width = spaces * SPACE_WIDTH + (spaces - 1).max(0) * CHAR_HORIZON_PADDING;
    let mut bold = false;
    let mut _italic = false;
    let mut fmt = false;
    let mut outputs = Vec::new();
    let mut cached = String::new();
    
    for ch in line.chars() {
        if width >= spaces_width || ch == '\n' {
            outputs.push(cached);
            cached = String::new();
            width = 0;
            if ch == '\n' {
                continue;
            }
        }
        
        if ch == '§' {
            fmt = true;
        } else if fmt {
            fmt = false;
            match ch {
                'l' => bold = true,
                'o' => _italic = true,
                'r' => {
                    bold = false;
                    _italic = false;
                }
                _ => {}
            }
        } else {
            width += get_char_width(&ch.to_string(), bold) + CHAR_HORIZON_PADDING;
        }
        cached.push(ch);
    }
    
    if !cached.trim().is_empty() {
        outputs.push(cached);
    }
    outputs
}

pub fn align_any_and_get_diff(text: &str, spaces: i32, prev_diff: i32) -> (String, i32) {
    let width = get_line_width(text);
    let spaces_left = spaces * SPACE_WIDTH - width;
    if spaces_left < 0 {
        return (String::new(), spaces_left);
    }
    get_specific_length_spaces_and_diff(spaces_left, prev_diff)
}

pub fn align_any(text: &str, spaces: i32) -> String {
    align_any_and_get_diff(text, spaces, 0).0
}

pub fn align_left(text: &str, spaces: i32) -> String {
    format!("{}{}", text, align_any(text, spaces))
}

pub fn align_left_and_get_diff(text: &str, spaces: i32, prev_diff: i32) -> (String, i32) {
    let (t, diff) = align_any_and_get_diff(text, spaces, prev_diff);
    (format!("{}{}", text, t), diff)
}

pub fn align_right(text: &str, spaces: i32) -> String {
    format!("{}{}", align_any(text, spaces), text)
}

pub fn align_right_and_get_diff(text: &str, spaces: i32, prev_diff: i32) -> (String, i32) {
    let (t, diff) = align_any_and_get_diff(text, spaces, prev_diff);
    (format!("{}{}", t, text), diff)
}

pub fn align_center(text: &str, spaces: i32) -> String {
    let textlen = get_line_width(text);
    let rest = spaces * SPACE_WIDTH - textlen;
    format!(
        "{}{}{}",
        get_specific_length_spaces(rest / 2),
        text,
        get_specific_length_spaces((rest as f64 / 2.0).round() as i32)
    )
}

pub fn align_simple(args: &[AlignArg]) -> String {
    let mut string = String::new();
    let mut diff = 0;
    
    for arg in args {
        match arg {
            AlignArg::Text(text) => {
                string.push_str(text);
            }
            AlignArg::LeftAlign(text, spaces) => {
                let (s, d) = align_left_and_get_diff(text, *spaces, -diff);
                string.push_str(&s);
                diff = d;
            }
            AlignArg::RightAlign(text, spaces) => {
                let (s, d) = align_right_and_get_diff(text, *spaces, -diff);
                string.push_str(&s);
                diff = d;
            }
        }
    }
    string
}

#[derive(Debug, Clone)]
pub enum AlignArg {
    Text(String),
    LeftAlign(String, i32),
    RightAlign(String, i32),
}

pub fn yield_chars_and_length(line: &str) -> Vec<(char, i32)> {
    let mut result = Vec::new();
    let mut width = 0;
    let mut bold = false;
    let mut _italic = false;
    let mut fmt = false;
    
    for ch in line.chars() {
        if ch == '§' {
            fmt = true;
        } else if fmt {
            fmt = false;
            match ch {
                'l' => bold = true,
                'o' => _italic = true,
                'r' => {
                    bold = false;
                    _italic = false;
                }
                _ => {}
            }
        } else {
            width += get_char_width(&ch.to_string(), bold) + ITALIC_CHAR_HORIZON_PADDING;
        }
        result.push((ch, width));
    }
    result
}