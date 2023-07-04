use std::fmt::Display;

pub struct TableFormatter {
    fields: Box<[(String, usize)]>,
    buffer: String,
}

pub struct Row<'a> {
    pos: usize,
    fmt: &'a mut TableFormatter,
}

impl Row<'_> {
    pub fn add(mut self, expr: impl Display) -> Self {
        use std::fmt::Write;
        let fields = &self.fmt.fields;
        assert!(self.pos < fields.len());
        let width = fields[self.pos].1;
        write!(self.fmt.buffer, "{expr: >width$}").expect("write to string");
        self.pos += 1;
        self
    }
}

impl TableFormatter {
    pub fn new(fields: impl IntoIterator<Item=(impl Into<String>, usize)>) -> Self {
        Self {
            fields:
                fields
                .into_iter()
                .map(|(name, width)| {
                    let name = name.into();
                    assert!(name.len() < width);
                    (name, width)
                })
                .collect(),
            buffer: String::new(),
        }
    }

    pub fn header(&mut self) -> impl Display + '_ {
        use std::fmt::Write;
        self.buffer.clear();
        let mut total_width = 0;
        for (expr, width) in self.fields.iter() {
            write!(self.buffer, "{expr: >width$}").expect("write to string");
            total_width += width;
        }
        write!(self.buffer, "\n{:=>total_width$}", "").expect("write to string");
        &self.buffer
    }

    pub fn print_header(&mut self) {
        println!("{}", self.header());
    }

    pub fn add(&mut self, f: impl FnOnce(Row) -> Row) -> impl Display + '_ {
        self.buffer.clear();
        let row = Row { pos: 0, fmt: self };
        let row = f(row);
        assert!(row.pos == self.fields.len());
        &self.buffer
    }
}

macro_rules! row {
    ($formatter:expr, $($field:expr),*) => {{
        println!("{}", $formatter.add(|row|
            row
            $(.add(&$field))*
        ));
    }}
}

pub(crate) use row;
