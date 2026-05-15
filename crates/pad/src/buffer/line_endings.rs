#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
}

impl LineEnding {
    pub fn detect(text: &str) -> Self {
        let crlf = text.matches("\r\n").count();
        let mut lone_cr = 0usize;
        let mut lone_lf = 0usize;
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\r' if i + 1 < bytes.len() && bytes[i + 1] == b'\n' => {
                    i += 2;
                    continue;
                }
                b'\r' => lone_cr += 1,
                b'\n' => lone_lf += 1,
                _ => {}
            }
            i += 1;
        }
        if crlf == 0 && lone_cr == 0 && lone_lf == 0 {
            return LineEnding::Lf;
        }
        if crlf >= lone_cr.max(lone_lf) {
            LineEnding::Crlf
        } else if lone_cr > lone_lf {
            LineEnding::Cr
        } else {
            LineEnding::Lf
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }
}
