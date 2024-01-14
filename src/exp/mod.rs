use chrono::{Datelike, NaiveDate};
use itertools::Itertools;

// struct Ledger {}

#[derive(Debug)]
pub struct File {
    filename: String,
    entries: Vec<Entry>,
}

fn parse_root(content: &str) -> Vec<Entry> {
    let mut entries = vec![];
    let mut remaining = content;
    while !remaining.is_empty() {
        let (entry, tmp) = parse_entry(remaining);
        remaining = tmp;
        entries.push(entry);
    }
    entries
}

// panics if content is empty
fn parse_entry(content: &str) -> (Entry, &str) {
    assert!(!content.is_empty());

    // What are the options now:
    // pragma (include / option / plugin / pushtag poptag pushmeta popmeta)
    // empty line
    // YYYY-MM-DD
    // include "<file>"
    // option "key" "value"
    // plugin "<plugin name>"
    // ; comment
    // unrecognized line

    // We already verified the content is not empty above with an assert
    match content.chars().next().unwrap() {
        '\n' => {
            // TODO: figure out if this can be done nicer
            let (line, remaining) = split_at_newline(content);
            (Entry::IgnoredLine(line.into()), remaining)
        }
        '*' | ':' | '#' | '!' | '&' | '?' | '%' => {
            let (line, remaining) = split_at_newline(content);
            (Entry::IgnoredLine(line.into()), remaining)
        }
        ';' => parse_comments(content),
        c if c.is_numeric() => parse_directive(content),
        // TODO: handle pragma entries
        _ => {
            let (line, remaining) = split_at_newline(content);
            (Entry::InvalidLine(line.to_string()), remaining)
        }
    }
}

// We expect that first character is comment here.
fn parse_comments(content: &str) -> (Entry, &str) {
    let mut comments = vec![];
    let mut r = content;
    loop {
        match r.chars().next() {
            Some(c) if c == ';' => {
                let (line, r1) = split_at_newline(r);
                r = r1;
                comments.push(line.into());
            }
            Some(c) if c.is_numeric() => {
                // TODO: optimize this clone
                match parse_directive_opt(r, comments.clone()) {
                    Some((d, r1)) => return (Entry::Directive(d), r1),
                    None => break,
                }
            }
            _ => break,
        }
    }
    (Entry::CommentBlock(comments), r)
}

fn parse_directive(content: &str) -> (Entry, &str) {
    match parse_directive_opt(content, vec![]) {
        None => {
            let (line, remaining) = split_at_newline(content);
            (Entry::InvalidLine(line.to_string()), remaining)
        }
        Some((d, r)) => (Entry::Directive(d), r),
    }
}

fn parse_directive_opt(content: &str, pre_comments: Vec<String>) -> Option<(Directive, &str)> {
    let (date, r) = read_date(content)?;

    let (ws1, r) = read_ws1(r)?;

    let (dir_type, r) = read_while(r, |c| c.is_alphabetic() || c == '*');

    match dir_type {
        "open" => {
            let (ws2, r) = read_ws1(r)?;
            let (account, r) = read_account(r)?;

            let (ws3, comment, r) = read_opt_inline_comment(r)?;

            Some((
                Directive::Open(Open {
                    pre_comments,
                    date,
                    ws1: ws1.into(),
                    ws2: ws2.into(),
                    account: account.into(),
                    ws3: ws3.into(),
                    inline_comment: comment.into(),
                }),
                r,
            ))
        }
        "txn" | "*" => {
            // let (ws2, r1) = read_ws1(r)?;

            let (ws2, comment, r) = read_opt_inline_comment(r)?;

            Some((
                Directive::Transaction(Transaction {
                    pre_comments,
                    date,
                    ws1: ws1.into(),
                    typ: dir_type.into(),
                    ws2: ws2.into(),
                    narration: None,
                    inline_comment: comment.into(),
                }),
                r,
            ))
        }
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    /// At least one whitespace followed by character that is not an end of line.
    Whitespace,
    /// optional whitespaces followed by end of line or end of file
    WhitespaceAndEnd,
    Comment,
    StringLit,
    Invalid,
}

fn read_token(content: &str) -> (Token, &str, &str) {
    let mut chars = content.char_indices();
    match chars.next() {
        None => return (Token::WhitespaceAndEnd, "", ""),
        Some((_, c)) if c == ' ' || c == '\t' => {
            for (p, c) in chars {
                if c == ' ' || c == '\t' {
                    continue;
                }
                if c == '\n' {
                    let (a, b) = content.split_at(p + 1);
                    return (Token::WhitespaceAndEnd, a, b);
                }
                let (a, b) = content.split_at(p);
                return (Token::Whitespace, a, b);
            }
            (Token::WhitespaceAndEnd, content, "")
        }
        Some((_, c)) if c == ';' => {
            let (a, b) = split_at_newline(content);
            (Token::Comment, a, b)
        }
        // TODO: add support for escaping the string
        Some((_, c)) if c == '"' => {
            for (p, c) in chars {
                if c == '"' {
                    let (a, b) = content.split_at(p + 1);
                    return (Token::StringLit, a, b);
                }
            }
            let (a, b) = split_at_newline(content);
            (Token::Invalid, a, b)
        }
        _ => todo!(),
    }
}

fn read_opt_inline_comment(content: &str) -> Option<(&str, &str, &str)> {
    // TODO: this could be faster
    let (line, r) = split_at_newline(content);
    let (ws, comment) = read_ws(line);
    let mut chars = comment.chars();
    match chars.next() {
        None | Some('\n') => Some((line, "", r)),
        Some(';') => Some((ws, comment, r)),
        _ => None,
    }
}

fn read_account(content: &str) -> Option<(&str, &str)> {
    // Each component:
    // - starts with capital letter or number
    // - contains letters, numbers or dashes
    let (a, r) = read_while(content, |c| c.is_alphanumeric() || c == '-' || c == ':');
    // TODO: verify that this is a proper account
    Some((a, r))
}

fn read_while<F: Fn(char) -> bool>(content: &str, f: F) -> (&str, &str) {
    let chars = content.char_indices();
    for (p, c) in chars {
        if !f(c) {
            return content.split_at(p);
        }
    }
    (content, "")
}

fn read_ws(content: &str) -> (&str, &str) {
    let chars = content.char_indices();
    for (p, c) in chars {
        if c != ' ' && c != '\t' {
            return content.split_at(p);
        }
    }
    (content, "")
}

// At least one whitespace
// TODO: add tests
fn read_ws1(content: &str) -> Option<(&str, &str)> {
    if content.is_empty() {
        return None;
    }
    let chars = content.char_indices();
    for (p, c) in chars {
        if c != ' ' && c != '\t' {
            if p == 0 {
                return None;
            }
            return Some(content.split_at(p));
        }
    }
    Some((content, ""))
}

fn read_date(content: &str) -> Option<(Date, &str)> {
    let (year, r) = read_number(content)?;
    if year.len() != 4 {
        return None;
    }
    let (sep1, r) = read_char(r)?;
    let sep = DateSeparator::try_from_char(sep1)?;
    let (month, r) = read_number(r)?;
    if month.len() != 2 {
        return None;
    }
    let (sep2, r) = read_char(r)?;
    if sep1 != sep2 {
        return None;
    }
    let (day, r) = read_number(r)?;
    if day.len() != 2 {
        return None;
    }
    Some((
        Date::from_ymd_with_sep(
            year.parse::<i32>().ok()?,
            month.parse::<u32>().ok()?,
            day.parse::<u32>().ok()?,
            sep,
        )?,
        r,
    ))
}

fn read_char(content: &str) -> Option<(char, &str)> {
    // TODO find if we can make it faster
    let mut chars = content.char_indices();
    let (_, ch) = chars.next()?;
    match chars.next() {
        None => Some((ch, "")),
        Some((p, _)) => Some((ch, content.split_at(p).1)),
    }
}

fn read_number(content: &str) -> Option<(&str, &str)> {
    let chars = content.char_indices();
    for (p, c) in chars {
        if !c.is_numeric() {
            if p == 0 {
                return None;
            }
            return Some(content.split_at(p));
        }
    }
    if content.is_empty() {
        return None;
    }
    Some((content, ""))
}

fn split_at_newline(content: &str) -> (&str, &str) {
    match content.find('\n') {
        Some(pos) => content.split_at(pos + 1),
        None => (content, ""),
    }
}

impl File {
    pub fn parse(filename: impl Into<String>, content: &str) -> anyhow::Result<File> {
        Ok(File {
            filename: filename.into(),
            entries: parse_root(content),
        })
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        self.entries.iter().map(|e| e.to_string()).join("")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Entry {
    /// A directive (that starts with a date).
    Directive(Directive),
    /// An entry that does not start with a date.
    Pragma(Pragma),
    /// A comment block not attached to any directive.
    CommentBlock(Vec<String>),
    /// A line starts with one of the ignored characters. '*', ':', '#', '!', '&', '?', '%'.
    IgnoredLine(String),

    InvalidLine(String),
}

impl Entry {
    pub fn as_directive(&self) -> Option<&Directive> {
        if let Entry::Directive(d) = self {
            return Some(d);
        }
        None
    }

    pub fn as_invalid_line(&self) -> Option<&String> {
        if let Entry::InvalidLine(l) = self {
            return Some(l);
        }
        None
    }

    pub fn as_open(&self) -> Option<&Open> {
        self.as_directive().and_then(Directive::as_open)
    }

    pub fn as_transaction(&self) -> Option<&Transaction> {
        self.as_directive().and_then(Directive::as_transaction)
    }
}

impl ToString for Entry {
    fn to_string(&self) -> String {
        match self {
            Entry::Directive(d) => d.to_string(),
            Entry::Pragma(_) => todo!(),
            Entry::CommentBlock(c) => c.iter().join(""),
            Entry::IgnoredLine(l) => l.into(),
            Entry::InvalidLine(l) => l.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Directive {
    Transaction(Transaction),
    Open(Open),
    Close(Close),
}

impl Directive {
    pub fn as_open(&self) -> Option<&Open> {
        if let Directive::Open(o) = self {
            return Some(o);
        }
        None
    }

    pub fn as_transaction(&self) -> Option<&Transaction> {
        if let Directive::Transaction(t) = self {
            return Some(t);
        }
        None
    }
}

impl ToString for Directive {
    fn to_string(&self) -> String {
        match self {
            Directive::Transaction(t) => t.to_string(),
            Directive::Open(d) => d.to_string(),
            Directive::Close(d) => d.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Account {
    v: String,
}

impl PartialEq<Account> for &str {
    fn eq(&self, other: &Account) -> bool {
        *self == other.v
    }
}

impl From<&str> for Account {
    fn from(value: &str) -> Self {
        Account { v: value.into() }
    }
}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.v)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pre_comments: Vec<String>,
    // date ws1 (txn|*) ws2 ?(?(payee ws3) (narration ws4)) (tag|link)* ?inline_comment
    date: Date,
    ws1: String,
    // txn || *
    typ: String,
    ws2: String,
    narration: Option<String>,
    inline_comment: String,
}

impl Transaction {
    pub fn date(&self) -> &Date {
        &self.date
    }
    pub fn narration(&self) -> &Option<String> {
        &self.narration
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.pre_comments.join(""),
            self.date.to_string(),
            self.ws1,
            self.typ,
            self.ws2,
            self.inline_comment
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Open {
    pre_comments: Vec<String>,
    // date ws1 open ws2 account ws3
    date: Date,
    ws1: String,
    ws2: String,
    account: Account,
    ws3: String,
    inline_comment: String,
}

impl Open {
    pub fn date(&self) -> &Date {
        &self.date
    }
    pub fn account(&self) -> &Account {
        &self.account
    }
}

impl ToString for Open {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.pre_comments.join(""),
            self.date.to_string(),
            self.ws1,
            "open",
            self.ws2,
            self.account,
            self.ws3,
            self.inline_comment
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Close {
    node: String,
}

impl ToString for Close {
    fn to_string(&self) -> String {
        self.node.clone()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DateSeparator {
    Dash,
    Slash,
}

impl DateSeparator {
    pub fn to_char(&self) -> char {
        match self {
            DateSeparator::Dash => '-',
            DateSeparator::Slash => '/',
        }
    }

    pub fn try_from_char(c: char) -> Option<DateSeparator> {
        if c == '-' {
            return Some(DateSeparator::Dash);
        }
        if c == '/' {
            return Some(DateSeparator::Slash);
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Date {
    // TODO: We should be able to encode the date and the separator into on u32.
    d: NaiveDate,
    sep: DateSeparator,
}

impl Date {
    // Creates the date with default dash as separator
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<Self> {
        Self::from_ymd_with_sep(year, month, day, DateSeparator::Dash)
    }

    pub fn from_ymd_with_sep(year: i32, month: u32, day: u32, sep: DateSeparator) -> Option<Self> {
        Some(Self {
            d: NaiveDate::from_ymd_opt(year, month, day)?,
            sep,
        })
    }

    pub fn naive(&self) -> NaiveDate {
        self.d
    }
}

impl ToString for Date {
    fn to_string(&self) -> String {
        format!(
            "{:04}{3}{:02}{3}{:02}",
            self.d.year(),
            self.d.month(),
            self.d.day(),
            self.sep.to_char(),
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Pragma {}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[googletest::test]
    fn test_split_at_newline() {
        expect_that!(split_at_newline(""), eq(("", "")));
        expect_that!(split_at_newline("a"), eq(("a", "")));
        expect_that!(split_at_newline("a\n"), eq(("a\n", "")));
        expect_that!(split_at_newline("a\nb"), eq(("a\n", "b")));
    }

    #[googletest::test]
    fn read_ws_consumes_all_whitespaces() {
        expect_that!(read_ws(""), eq(("", "")));
        expect_that!(read_ws(" "), eq((" ", "")));
        expect_that!(read_ws("a"), eq(("", "a")));
        expect_that!(read_ws(" a"), eq((" ", "a")));
        expect_that!(read_ws(" \ta"), eq((" \t", "a")));
    }

    #[googletest::test]
    fn test_read_date() {
        expect_that!(read_date(""), eq(None));
        expect_that!(read_date("2000-01-2"), eq(None));
        expect_that!(read_date("2022-1-02"), eq(None));
        expect_that!(read_date("2022/01-02"), eq(None));
        expect_that!(read_date("2022-01/02"), eq(None));
        expect_that!(
            read_date("2000-01-02"),
            eq(Some((Date::from_ymd(2000, 1, 2).unwrap(), "")))
        );
        expect_that!(
            read_date("2000-01-02 open"),
            eq(Some((Date::from_ymd(2000, 1, 2).unwrap(), " open")))
        );
        expect_that!(
            read_date("2000/01/02"),
            eq(Some((
                Date::from_ymd_with_sep(2000, 1, 2, DateSeparator::Slash).unwrap(),
                ""
            )))
        );
    }

    #[googletest::test]
    fn test_read_char() {
        expect_that!(read_char(""), eq(None));
        expect_that!(read_char("a"), eq(Some(('a', ""))));
        expect_that!(read_char("ab"), eq(Some(('a', "b"))));
    }

    #[googletest::test]
    fn test_read_number() {
        expect_that!(read_number(""), eq(None));
        expect_that!(read_number("a"), eq(None));
        expect_that!(read_number("1a"), eq(Some(("1", "a"))));
        expect_that!(read_number("1"), eq(Some(("1", ""))));
        expect_that!(read_number("02-"), eq(Some(("02", "-"))));
    }

    #[googletest::test]
    fn test_read_token() {
        expect_that!(read_token(""), eq((Token::WhitespaceAndEnd, "", "")));
        expect_that!(read_token(" "), eq((Token::WhitespaceAndEnd, " ", "")));
        expect_that!(read_token(" \n"), eq((Token::WhitespaceAndEnd, " \n", "")));
        expect_that!(
            read_token(" \na"),
            eq((Token::WhitespaceAndEnd, " \n", "a"))
        );

        expect_that!(read_token(" a"), eq((Token::Whitespace, " ", "a")));
        expect_that!(read_token("\ta"), eq((Token::Whitespace, "\t", "a")));
        expect_that!(read_token("\t a"), eq((Token::Whitespace, "\t ", "a")));

        expect_that!(read_token(";a"), eq((Token::Comment, ";a", "")));
        expect_that!(read_token(";a\n"), eq((Token::Comment, ";a\n", "")));
        expect_that!(read_token(";a\nb"), eq((Token::Comment, ";a\n", "b")));

        expect_that!(read_token("\"\""), eq((Token::StringLit, "\"\"", "")));
        expect_that!(read_token("\"a\""), eq((Token::StringLit, "\"a\"", "")));
        expect_that!(read_token("\"a\"b"), eq((Token::StringLit, "\"a\"", "b")));
        // Multiline string
        expect_that!(read_token("\"a\n\""), eq((Token::StringLit, "\"a\n\"", "")));
        // Not closed quote
        expect_that!(read_token("\"a\nb"), eq((Token::Invalid, "\"a\n", "b")));
    }
}
