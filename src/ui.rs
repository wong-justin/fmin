

pub struct Pt(usize, usize);

pub struct Rect {
    pub tl: Pt,
    pub br: Pt,
}

#[derive(Default)]
pub struct Layout {
    pub W: usize,
    pub H: usize,
    pub row1_start: usize,
    pub row1_end: usize,
    pub col1_start: usize,
    pub col1_end: usize,
    pub col2_start: usize,
    pub col2_end: usize,
    pub col3_start: usize,
    pub col3_end: usize,
    pub row2_start: usize,
    pub row2_end: usize,
    pub row3_start: usize,
    pub row3_end: usize,
    pub row4_start: usize,
    pub row4_end: usize,

    pub list_min_pos: usize,
    pub list_max_pos: usize,
}

/*
   border padding 1 

  TitleLine
  -----------------------------
  ExpandH      | Fixed  | Fixed   
  -----------------------------
  List         |        |
  ExpandV      |        | 
               |        |
  ----------------------------
  FooterLine 


*/
impl Layout {

    pub fn resize(&mut self, w: u16, h: u16) {
        self.W = w as usize;
        self.H = h as usize;
        self.row1_start = 1;
        self.row1_end = 1;
        self.row2_start = 3;
        self.row2_end = 3;
        self.row3_start = 5;
        self.row3_end = self.H - 4;
        self.row4_start = self.H - 2;
        self.row4_end = self.H - 2;
        self.col3_end = self.W - 2;
        self.col3_start = &self.col3_end - 17 + 1;
        self.col2_end = &self.col3_start - 2;
        self.col2_start = &self.col2_end - 9 + 1;
        self.col1_end = &self.col2_start - 2;
        self.col1_start = 1;

        self.list_max_pos = self.H - 6 + self.list_min_pos;
    }

    pub fn reset_list_pos(&mut self) {
        self.list_min_pos = 0;
        self.list_max_pos = self.H - 6
    }
}


