use crate::define::CHAR_HORIZON_PADDING;
use crate::align::{get_line_width, get_char_width};
use crate::utils::solve_xy;

// Check if all numbers have the same parity
pub fn check_same_parity(c: &[i32]) -> bool {
    if c.is_empty() {
        return true;
    }
    let p = c[0] & 1;
    c.iter().all(|&ci| (ci & 1) == p)
}

// Resolve padding values
pub fn resolve(c: &[i32]) -> Option<Vec<(i32, i32)>> {
    if c.is_empty() {
        return None;
    }

    if !check_same_parity(c) {
        return None;
    }
    
    let parity = c[0] & 1;
    let mut width = *c.iter().max().unwrap();
    if (width & 1) != parity {
        width += 1;
    }

    loop {
        let mut res = Vec::new();
        let mut ok = true;
        
        for &ci in c {
            let di = width - ci;
            if let Some((x, y)) = solve_xy(CHAR_HORIZON_PADDING, get_line_width("§l ") + CHAR_HORIZON_PADDING, di) {
                // Remove the assertion that was causing the panic
                res.push((x, y));
            } else {
                ok = false;
                break;
            }
        }
        
        if ok {
            return Some(res);
        } else {
            width += 2;
        }
    }
}

pub fn pad(texts: &[String]) -> Vec<String> {
    let cs: Vec<i32> = texts.iter().map(|t| get_line_width(t)).collect();
    let res = resolve(&cs).expect("Failed to resolve padding");
    let pads: Vec<String> = res.iter().map(|(ns, nb)| {
        let mut pad = "§r".to_string();
        if *ns > 0 {
            pad.push_str(&" ".repeat(*ns as usize));
        }
        if *nb > 0 {
            pad.push_str(&format!("§l{}§r", " ".repeat(*nb as usize)));
        }
        pad
    }).collect();
    
    texts.iter().zip(pads.iter()).map(|(t, p)| format!("{}{}", t, p)).collect()
}

pub struct Padder<F> 
where
    F: Fn(&[String]) -> Vec<String>,
{
    pending_lines: Vec<String>,
    padded: Vec<String>,
    pad_i: usize,
    pad_mark: String,
    pad_fn: F,
}

impl<F> Padder<F>
where
    F: Fn(&[String]) -> Vec<String>,
{
    pub fn new(text_lines: &str, pad_fn: F) -> Self {
        let lines: Vec<String> = text_lines.lines().map(|s| s.to_string()).collect();
        let pad_i = 1;
        let pad_mark = format!("(pad{})", pad_i);
        
        Self {
            pending_lines: lines,
            padded: vec![String::new(); text_lines.lines().count()],
            pad_i,
            pad_mark,
            pad_fn,
        }
    }

    fn step(&mut self) {
        assert!(!self.all_done());
        let mut match_list = Vec::new();
        let mut match_index = Vec::new();
        let mut updates = Vec::new();
        
        // First pass: collect data without modifying
        for (i, (c, p)) in self.padded.iter().zip(self.pending_lines.iter()).enumerate() {
            if !p.contains(&self.pad_mark) {
                continue;
            }
            match_index.push(i);
            let parts: Vec<&str> = p.splitn(2, &self.pad_mark).collect();
            if parts.len() == 2 {
                let (t, r) = (parts[0], parts[1]);
                match_list.push(format!("{}{}", c, t));
                updates.push((i, r.to_string()));
            }
        }
        
        let out = (self.pad_fn)(&match_list);
        
        // Apply updates
        for (i, r) in updates {
            self.pending_lines[i] = r;
        }
        
        for (i, o) in match_index.iter().zip(out.iter()) {
            self.padded[*i] = o.clone();
        }
        
        self.pad_i += 1;
        self.pad_mark = format!("(pad{})", self.pad_i);
    }

    fn all_done(&self) -> bool {
        self.pending_lines.iter().all(|ln| !ln.contains(&self.pad_mark))
    }

    fn finish(&self) -> String {
        self.pending_lines.iter()
            .zip(self.padded.iter())
            .map(|(ln, t)| format!("{}{}", ln, t))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn execute(mut self) -> String {
        while !self.all_done() {
            self.step();
        }
        self.finish()
    }
}

pub fn pad_with_format(text: &str) -> String {
    Padder::new(text, pad).execute()
}

pub fn pad_with_length(length: i32, padder: &str, round: bool) -> String {
    let length = length + CHAR_HORIZON_PADDING;
    let char_width = get_char_width(padder, false) + CHAR_HORIZON_PADDING;
    if !round {
        padder.repeat((length / char_width) as usize)
    } else {
        padder.repeat(((length as f64 / char_width as f64).round()) as usize)
    }
}