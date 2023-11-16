/// Break up glossary into terms we want to have searchable.
///
/// Such as `to read, to write (something)` => `["to", "to read", "to", "to
/// write", "something", "to write (something)"]`.
pub(super) fn analyze(input: &str) -> AnalyzeGlossary<'_> {
    AnalyzeGlossary {
        input: input.trim(),
        base: 0,
        parens: vec![],
        o: 0,
    }
}

pub(super) struct AnalyzeGlossary<'a> {
    input: &'a str,
    base: usize,
    parens: Vec<usize>,
    o: usize,
}

const NUL: char = '\u{0000}';

impl<'a> AnalyzeGlossary<'a> {
    fn c(&self) -> char {
        let Some(c) = self.input[self.o..].chars().next() else {
            return NUL;
        };

        c
    }

    fn step(&mut self) {
        let c = self.c();

        if !matches!(c, NUL) {
            self.o += c.len_utf8();
        }
    }

    fn space(&mut self) {
        while matches!(self.c(), ' ') {
            self.step();
        }
    }

    fn until_term(&mut self) {
        while !matches!(self.c(), ' ' | ')' | ',' | NUL) {
            self.step();
        }
    }
}

impl<'a> Iterator for AnalyzeGlossary<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.space();

            if self.o == self.input.len() {
                return None;
            }

            match self.c() {
                ',' => {
                    self.step();

                    if self.parens.is_empty() {
                        self.space();
                        self.base = self.o;
                    }
                }
                '(' => {
                    self.step();
                    self.parens.push(self.o);
                }
                ')' => {
                    self.step();
                    self.parens.pop();
                    let start = self.parens.last().copied().unwrap_or(self.base);
                    return Some(&self.input[start..self.o]);
                }
                _ => {
                    self.until_term();
                    let start = self.parens.last().copied().unwrap_or(self.base);
                    return Some(&self.input[start..self.o]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            analyze("to read, to look (something (very) cool) sometimes").collect::<Vec<_>>(),
            vec![
                "to",
                "to read",
                "to",
                "to look",
                "something",
                "very",
                "something (very)",
                "something (very) cool",
                "to look (something (very) cool)",
                "to look (something (very) cool) sometimes"
            ]
        );
    }
}
