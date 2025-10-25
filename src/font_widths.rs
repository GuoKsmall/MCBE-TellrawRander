use std::collections::HashMap;

pub struct FontWidths {
    widths: HashMap<u32, u32>,
}

impl FontWidths {
    pub fn new() -> Self {
        let mut widths = HashMap::new();
        
        // ASCII character widths
        for i in 0..128 {
            let width = match i {
                32 => 4,  // Space
                33 => 2,  // !
                34 => 5,  // "
                35 => 6,  // #
                36 => 6,  // $
                37 => 6,  // %
                38 => 6,  // &
                39 => 2,  // '
                40 => 4,  // (
                41 => 4,  // )
                42 => 5,  // *
                43 => 6,  // +
                44 => 2,  // ,
                45 => 6,  // -
                46 => 2,  // .
                47 => 6,  // /
                48..=57 => 6, // Digits 0-9
                58 => 2,  // :
                59 => 2,  // ;
                60 => 5,  // <
                61 => 6,  // =
                62 => 5,  // >
                63 => 6,  // ?
                64 => 7,  // @
                65..=90 => 6,  // A-Z
                91 => 4,  // [
                92 => 6,  // \
                93 => 4,  // ]
                94 => 6,  // ^
                95 => 6,  // _
                96 => 3,  // `
                97..=122 => 6, // a-z
                123 => 4, // {
                124 => 2, // |
                125 => 4, // }
                126 => 7, // ~
                _ => 6,   // Default width
            };
            widths.insert(i, width);
        }
        
        Self { widths }
    }
    
    // Get character width by code
    pub fn get_width(&self, char_code: u32) -> u32 {
        self.widths.get(&char_code).copied().unwrap_or(6)
    }
}

impl Default for FontWidths {
    fn default() -> Self {
        Self::new()
    }
}
