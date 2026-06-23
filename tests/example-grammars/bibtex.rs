use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::prelude::*;
use rust_petitparser_macros::grammar;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct BibTeXEntry {
    pub kind: String,
    pub key: String,
    pub fields: Vec<(String, String)>,
}

impl fmt::Display for BibTeXEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}{{{}", self.kind, self.key)?;
        for (key, value) in &self.fields {
            write!(f, ",\n\t{} = {}", key, value)?;
        }
        write!(f, "}}")
    }
}

#[grammar]
mod bibtex_grammar {
    fn start() -> impl Parser<Vec<BibTeXEntry>> {
        entries().end()
    }

    fn entries() -> impl Parser<Vec<BibTeXEntry>> {
        entry().star_sep(whitespace().star())
    }

    fn entry() -> impl Parser<BibTeXEntry> {
        seq6(
            type_token().trim(),
            char('{').trim(),
            cite_key().trim(),
            char(',').trim(),
            fields(),
            char('}').trim(),
        )
        .map(|(kind, _, key, _, fields, _)| BibTeXEntry { kind, key, fields })
    }

    fn fields() -> impl Parser<Vec<(String, String)>> {
        field().star_sep(char(',').trim())
    }

    fn field() -> impl Parser<(String, String)> {
        seq3(field_name().trim(), char('=').trim(), field_value())
            .map(|(name, _, value)| (name, value))
    }

    fn field_value() -> impl Parser<String> {
        choice3(
            field_value_in_quotes(),
            field_value_in_braces(),
            raw_string(),
        )
    }

    fn field_value_in_quotes() -> impl Parser<String> {
        seq3(char('"'), field_string_within_quotes(), char('"'))
            .input_with_message("quoted string expected".to_string())
    }

    fn field_string_within_quotes() -> impl Parser<Vec<()>> {
        choice2(field_char_within_quotes(), escape_char()).star()
    }

    fn field_char_within_quotes() -> impl Parser<()> {
        pattern("^\\\"").map(|_| ())
    }

    fn field_value_in_braces() -> impl Parser<String> {
        seq3(char('{'), field_string_within_braces(), char('}'))
            .input_with_message("braced string expected".to_string())
    }

    fn field_string_within_braces() -> impl Parser<Vec<()>> {
        choice3(
            field_char_within_braces(),
            escape_char(),
            seq3(char('{'), field_string_within_braces(), char('}')).map(|_| ()),
        )
        .star()
    }

    fn field_char_within_braces() -> impl Parser<()> {
        pattern("^\\{}").map(|_| ())
    }

    fn raw_string() -> impl Parser<String> {
        pattern("a-zA-Z0-9")
            .plus()
            .input_with_message("raw string expected".to_string())
    }

    fn type_token() -> impl Parser<String> {
        letter()
            .plus()
            .input_with_message("type expected".to_string())
            .skip_left(char('@'))
    }

    fn cite_key() -> impl Parser<String> {
        pattern("a-zA-Z0-9_:-")
            .plus()
            .input_with_message("citation key expected".to_string())
    }

    fn field_name() -> impl Parser<String> {
        pattern("a-zA-Z0-9_-")
            .plus()
            .input_with_message("field name expected".to_string())
    }

    fn escape_char() -> impl Parser<()> {
        seq2(char('\\'), any()).map(|_| ())
    }
}

fn sample_entry_text() -> &'static str {
    "@inproceedings{Reng10c,\n\
     \tTitle = \"Practical Dynamic Grammars for Dynamic Languages\",\n\
     \tAuthor = {Lukas Renggli and St\\'ephane Ducasse and Tudor G\\^irba and Oscar Nierstrasz},\n\
     \tMonth = jun,\n\
     \tYear = 2010,\n\
     \tUrl = {http://scg.unibe.ch/archive/papers/Reng10cDynamicGrammars.pdf}}"
}

#[gtest]
fn basic_parsing() {
    let p = BibtexGrammar::new();
    let input = sample_entry_text();
    assert_success!(
        p,
        input,
        &vec![BibTeXEntry {
            kind: "inproceedings".to_string(),
            key: "Reng10c".to_string(),
            fields: vec![
                (
                    "Title".to_string(),
                    "\"Practical Dynamic Grammars for Dynamic Languages\"".to_string()
                ),
                (
                    "Author".to_string(),
                    "{Lukas Renggli and St\\'ephane Ducasse and Tudor G\\^irba and Oscar Nierstrasz}"
                        .to_string()
                ),
                ("Month".to_string(), "jun".to_string()),
                ("Year".to_string(), "2010".to_string()),
                (
                    "Url".to_string(),
                    "{http://scg.unibe.ch/archive/papers/Reng10cDynamicGrammars.pdf}".to_string()
                ),
            ],
        }]
    );
}

#[gtest]
fn basic_serializing() {
    let p = BibtexGrammar::new();
    let input = sample_entry_text();
    let entries = p.parse(input).unwrap().value;
    assert_that!(entries[0].to_string(), eq(input));
}

#[gtest]
fn scg_bib_size_and_round_trip() {
    let body: String = ureq::get("https://raw.githubusercontent.com/scgbern/scgbib/main/scg.bib")
        .call()
        .expect("failed to fetch scg.bib")
        .body_mut()
        .read_to_string()
        .expect("failed to read scg.bib body");

    let p = BibtexGrammar::new();
    let entries = p.parse(&body).expect("failed to parse scg.bib").value;

    assert_that!(entries.len(), gt(9600));

    let renggli_count = entries
        .iter()
        .filter(|entry| {
            entry
                .fields
                .iter()
                .any(|(key, value)| key == "Author" && value.contains("Renggli"))
        })
        .count();
    assert_that!(renggli_count, gt(35));

    for entry in &entries {
        let round_tripped = p
            .parse(&entry.to_string())
            .expect("round-trip parse failed")
            .value;
        assert_that!(round_tripped.len(), eq(1));
        assert_that!(&round_tripped[0], eq(entry));
    }
}

#[gtest]
fn multiple_entries_separated_by_whitespace() {
    let p = BibtexGrammar::new();
    let input = "@book{a,\n\tTitle = short}\n\n@book{b,\n\tTitle = other}";
    assert_success!(
        p,
        input,
        &vec![
            BibTeXEntry {
                kind: "book".to_string(),
                key: "a".to_string(),
                fields: vec![("Title".to_string(), "short".to_string())],
            },
            BibTeXEntry {
                kind: "book".to_string(),
                key: "b".to_string(),
                fields: vec![("Title".to_string(), "other".to_string())],
            },
        ]
    );
}
