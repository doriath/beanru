use beanru::exp::*;
use googletest::prelude::*;

#[googletest::test]
fn parses_empty_file() -> anyhow::Result<()> {
    let input = "";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    expect_that!(*f.entries(), empty());
    Ok(())
}

#[googletest::test]
fn parses_unknown_lines() -> anyhow::Result<()> {
    let input = "some invalid line";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    expect_that!(*f.entries(), len(eq(1)));
    Ok(())
}

#[googletest::test]
fn parses_two_unknown_lines() -> anyhow::Result<()> {
    let input = "line1\nline2";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    expect_that!(*f.entries(), len(eq(2)));
    Ok(())
}

#[googletest::test]
fn parses_open_directive() -> anyhow::Result<()> {
    let input = "2023-01-02 open Assets:Test";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    assert_that!(*f.entries(), len(eq(1)));
    let o = f.entries()[0]
        .as_open()
        .expect("entry is not an open directive");
    expect_that!(*o.date(), eq(Date::from_ymd(2023, 1, 2).unwrap()));
    expect_that!(*o.account(), eq("Assets:Test"));
    Ok(())
}

#[googletest::test]
fn parses_open_directive_with_invalid_last_term() -> anyhow::Result<()> {
    let input = "2023-01-02 open Assets:Test invalid";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    assert_that!(*f.entries(), len(eq(1)));
    expect_that!(
        *f.entries(),
        elements_are![eq(Entry::InvalidLine(input.into()))]
    );
    Ok(())
}

#[googletest::test]
fn parses_open_directive_with_inline_comment() -> anyhow::Result<()> {
    let input = "2023-01-02 open Assets:Test ;asdf";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    assert_that!(*f.entries(), len(eq(1)));
    let o = f.entries()[0]
        .as_open()
        .expect("entry is not an open directive");
    expect_that!(*o.date(), eq(Date::from_ymd(2023, 1, 2).unwrap()));
    expect_that!(*o.account(), eq("Assets:Test"));
    Ok(())
}

#[googletest::test]
fn parses_open_directive_with_pre_comment() -> anyhow::Result<()> {
    let input = ";comment\n2023-01-02 open Assets:Test";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    assert_that!(*f.entries(), len(eq(1)));
    let o = f.entries()[0]
        .as_open()
        .expect("entry is not an open directive");
    expect_that!(*o.date(), eq(Date::from_ymd(2023, 1, 2).unwrap()));
    expect_that!(*o.account(), eq("Assets:Test"));
    Ok(())
}

#[googletest::test]
fn parses_transaction() -> anyhow::Result<()> {
    let input = "2023-01-02 txn";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    assert_that!(*f.entries(), len(eq(1)));
    let t = f.entries()[0]
        .as_transaction()
        .expect("entry is not an transaction directive");
    Ok(())
}

// #[googletest::test]
// fn parses_transaction_with_narration() -> anyhow::Result<()> {
//     let input = "2023-01-02 txn \"narration\"";
//     let f = File::parse("test.beancount", input)?;
//     expect_that!(f.filename(), eq("test.beancount"));
//     expect_that!(f.to_string(), eq(input));
//     assert_that!(*f.entries(), len(eq(1)));
//     let t = f.entries()[0]
//         .as_transaction()
//         .expect("entry is not an transaction directive");
//     Ok(())
// }

#[googletest::test]
fn parses_comments() -> anyhow::Result<()> {
    let input = ";comment1\n;comment2\n";
    let f = File::parse("test.beancount", input)?;
    expect_that!(f.filename(), eq("test.beancount"));
    expect_that!(f.to_string(), eq(input));
    assert_that!(*f.entries(), len(eq(1)));
    Ok(())
}

#[googletest::test]
fn test_date_to_string() {
    expect_that!(
        Date::from_ymd(2000, 1, 2).unwrap().to_string(),
        eq("2000-01-02")
    );
    expect_that!(
        Date::from_ymd(2000, 10, 11).unwrap().to_string(),
        eq("2000-10-11")
    );
}
