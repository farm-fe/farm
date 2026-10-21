use codemap::Spanned;

use super::AstExpr;

#[derive(Debug, Clone)]
pub struct Interpolation {
    pub contents: Vec<InterpolationPart>,
}

impl Interpolation {
    pub fn new() -> Self {
        Self {
            contents: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn new_with_expr(e: Spanned<AstExpr>) -> Self {
        Self {
            contents: vec![InterpolationPart::Expr(e)],
        }
    }

    pub fn new_plain(s: String) -> Self {
        Self {
            contents: vec![InterpolationPart::String(s)],
        }
    }

    pub fn add_expr(&mut self, expr: Spanned<AstExpr>) {
        self.contents.push(InterpolationPart::Expr(expr));
    }

    pub fn add_string(&mut self, s: String) {
        match self.contents.last_mut() {
            Some(InterpolationPart::String(existing)) => existing.push_str(&s),
            _ => self.contents.push(InterpolationPart::String(s)),
        }
    }

    pub fn add_char(&mut self, c: char) {
        match self.contents.last_mut() {
            Some(InterpolationPart::String(existing)) => existing.push(c),
            _ => self.contents.push(InterpolationPart::String(c.to_string())),
        }
    }

    pub fn add_interpolation(&mut self, mut other: Self) {
        self.contents.append(&mut other.contents);
    }

    pub fn initial_plain(&self) -> &str {
        match self.contents.first() {
            Some(InterpolationPart::String(s)) => s,
            _ => "",
        }
    }

    pub fn as_plain(&self) -> Option<&str> {
        if self.contents.is_empty() {
            Some("")
        } else if self.contents.len() > 1 {
            None
        } else {
            match self.contents.first()? {
                InterpolationPart::String(s) => Some(s),
                InterpolationPart::Expr(..) => None,
            }
        }
    }

    pub fn trailing_string(&self) -> &str {
        match self.contents.last() {
            Some(InterpolationPart::String(s)) => s,
            Some(InterpolationPart::Expr(..)) | None => "",
        }
    }
}

#[derive(Debug, Clone)]
pub enum InterpolationPart {
    String(String),
    Expr(Spanned<AstExpr>),
}
