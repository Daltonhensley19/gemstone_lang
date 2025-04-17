#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line_num: usize,
    pub col_num: usize,
}

impl Span {
    pub fn new() -> Self {
        Self {
            line_num: 1,
            col_num: 1,
        }
    }

    pub fn new_with(line_num: usize, col_num: usize) -> Self {
        Self { line_num, col_num }
    }

    pub fn reset_col_num(&mut self) {
        self.col_num = 0;
    }

    pub fn incre_line_num(&mut self) {
        self.line_num += 1;
    }

    pub fn incre_col_num(&mut self) {
        self.col_num += 1;
    }

    pub fn incre_col_num_by(&mut self, offset: usize) {
        self.col_num += offset;
    }

    pub fn incre_line_num_by(&mut self, offset: usize) {
        self.line_num += offset;
    }

    pub fn decre_line_num(&mut self) {
        self.line_num -= 1;
    }

    pub fn decre_col_num(&mut self) {
        self.col_num -= 1;
    }

    pub fn incre_from_char(&mut self, chr: char) {
        match chr {
            '\n' => {
                self.incre_line_num();
                self.reset_col_num();
            }
            '\t' => {
                self.incre_col_num_by(4);
            }
            _ => {
                self.incre_col_num();
            }
        }
    }
}

