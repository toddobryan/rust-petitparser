use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::prelude::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct TabularDefinition {
    quote: Rc<dyn Parser<String>>,
    escape: Rc<dyn Parser<String>>,
    delimiter: Rc<dyn Parser<String>>,
    newline: Rc<dyn Parser<String>>,
}

impl Parser<Vec<Vec<String>>> for TabularDefinition {
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<Vec<String>>> {
        self.start().parse_on(context)
    }
}

impl TabularDefinition {
    pub fn csv() -> Self {
        TabularDefinition {
            quote: Rc::new(string("\"")),
            escape: Rc::new(string("\"\"").constant("\"".to_string())),
            delimiter: Rc::new(string(",")),
            newline: Rc::new(newline()),
        }
    }

    pub fn tsv() -> Self {
        TabularDefinition {
            quote: Rc::new(failure().map(|_| String::new())),
            escape: Rc::new(seq2(char('\\'), any()).map(|(_, value)| match value {
                't' => "\t".to_string(),
                'n' => "\n".to_string(),
                'r' => "\r".to_string(),
                c => c.to_string(),
            })),
            delimiter: Rc::new(string("\t")),
            newline: Rc::new(newline()),
        }
    }

    fn start(&self) -> impl Parser<Vec<Vec<String>>> {
        self.lines().end()
    }

    fn lines(&self) -> impl Parser<Vec<Vec<String>>> {
        self.records()
            .star_sep(self.newline.clone(), Trailing::Disallowed)
    }

    fn records(&self) -> impl Parser<Vec<String>> {
        self.field()
            .star_sep(self.delimiter.clone(), Trailing::Disallowed)
    }

    fn field(&self) -> impl Parser<String> {
        choice2(self.quoted_field(), self.plain_field())
    }

    fn quoted_field(&self) -> impl Parser<String> {
        self.quoted_field_content()
            .skip(self.quote.clone(), self.quote.clone())
    }

    fn quoted_field_content(&self) -> impl Parser<String> {
        self.quoted_field_char()
            .star()
            .map(|v: Vec<String>| v.concat())
    }

    fn quoted_field_char(&self) -> impl Parser<String> {
        choice2(
            self.escape.clone(),
            self.quote.clone().neg().map(|c: char| c.to_string()),
        )
    }

    fn plain_field(&self) -> impl Parser<String> {
        self.plain_field_content()
    }

    fn plain_field_content(&self) -> impl Parser<String> {
        self.plain_field_char()
            .star()
            .map(|v: Vec<String>| v.concat())
    }

    fn plain_field_char(&self) -> impl Parser<String> {
        choice2(
            self.escape.clone(),
            choice2(self.delimiter.clone(), self.newline.clone())
                .neg()
                .map(|c: char| c.to_string()),
        )
    }
}

fn row(fields: &[&str]) -> Vec<String> {
    fields.iter().map(|s| s.to_string()).collect()
}

#[gtest]
fn csv_basic_string() {
    let p = TabularDefinition::csv();
    assert_success!(p, "a", &vec![row(&["a"])], 1);
    assert_success!(p, "ab", &vec![row(&["ab"])], 2);
    assert_success!(p, "abc", &vec![row(&["abc"])], 3);
}

#[gtest]
fn csv_quoted_string() {
    let p = TabularDefinition::csv();
    assert_success!(p, "\"\"", &vec![row(&[""])], 2);
    assert_success!(p, "\"a\"", &vec![row(&["a"])], 3);
    assert_success!(p, "\"ab\"", &vec![row(&["ab"])], 4);
    assert_success!(p, "\"abc\"", &vec![row(&["abc"])], 5);
    assert_success!(p, "\"\"\"\"", &vec![row(&["\""])], 4);
}

#[gtest]
fn csv_fields() {
    let p = TabularDefinition::csv();
    assert_success!(p, "a,b", &vec![row(&["a", "b"])], 3);
    assert_success!(p, "a,b,c", &vec![row(&["a", "b", "c"])], 5);
}

#[gtest]
fn csv_fields_empty() {
    let p = TabularDefinition::csv();
    assert_success!(p, "", &vec![row(&[""])], 0);
    assert_success!(p, ",", &vec![row(&["", ""])], 1);
    assert_success!(p, ",,", &vec![row(&["", "", ""])], 2);
}

#[gtest]
fn csv_lines() {
    let p = TabularDefinition::csv();
    assert_success!(p, "a\nb", &vec![row(&["a"]), row(&["b"])], 3);
    assert_success!(
        p,
        "a\nb\nc",
        &vec![row(&["a"]), row(&["b"]), row(&["c"])],
        5
    );
    assert_success!(p, "\n", &vec![row(&[""]), row(&[""])], 1);
    assert_success!(p, "\n\n", &vec![row(&[""]), row(&[""]), row(&[""])], 2);
}

#[gtest]
fn tsv_basic_string() {
    let p = TabularDefinition::tsv();
    assert_success!(p, "a", &vec![row(&["a"])], 1);
    assert_success!(p, "ab", &vec![row(&["ab"])], 2);
    assert_success!(p, "abc", &vec![row(&["abc"])], 3);
}

#[gtest]
fn tsv_escaped_string() {
    let p = TabularDefinition::tsv();
    assert_success!(p, "\\t", &vec![row(&["\t"])], 2);
    assert_success!(p, "\\n", &vec![row(&["\n"])], 2);
    assert_success!(p, "\\r", &vec![row(&["\r"])], 2);
    assert_success!(p, "\\\\", &vec![row(&["\\"])], 2);
}

#[gtest]
fn tsv_fields() {
    let p = TabularDefinition::tsv();
    assert_success!(p, "a\tb", &vec![row(&["a", "b"])], 3);
    assert_success!(p, "a\tb\tc", &vec![row(&["a", "b", "c"])], 5);
}

#[gtest]
fn tsv_fields_empty() {
    let p = TabularDefinition::tsv();
    assert_success!(p, "", &vec![row(&[""])], 0);
    assert_success!(p, "\t", &vec![row(&["", ""])], 1);
    assert_success!(p, "\t\t", &vec![row(&["", "", ""])], 2);
}

#[gtest]
fn tsv_lines() {
    let p = TabularDefinition::tsv();
    assert_success!(p, "a\nb", &vec![row(&["a"]), row(&["b"])], 3);
    assert_success!(
        p,
        "a\nb\nc",
        &vec![row(&["a"]), row(&["b"]), row(&["c"])],
        5
    );
    assert_success!(p, "\n", &vec![row(&[""]), row(&[""])], 1);
    assert_success!(p, "\n\n", &vec![row(&[""]), row(&[""]), row(&[""])], 2);
}
