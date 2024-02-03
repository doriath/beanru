use crate::exp::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pre_comments: Vec<String>,
    // date ws1 (txn|*) ?(?(ws2 payee) (ws3 narration)) (wsx tag|link)* ws_last ?inline_comment
    date: Date,
    // txn || *
    // ws_pre_typ: String,
    typ: WithWS<String>,

    payee: Option<WithWS<String>>,
    narration: Option<WithWS<String>>,

    tags_and_links: Vec<WithWS<TagOrLink>>,

    ws_last: String,
    inline_comment: String,
    eol: String,
}

impl Transaction {
    pub fn date(&self) -> &Date {
        &self.date
    }
    pub fn narration(&self) -> Option<&String> {
        self.narration.as_ref().map(|n| &n.v)
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        let mut ret = format!(
            "{}{}{}",
            self.pre_comments.join(""),
            self.date.to_string(),
            self.typ.to_string()
        );
        if let Some(ref payee) = self.payee {
            ret.push_str(&payee.to_string());
        }
        if let Some(ref narration) = self.narration {
            ret.push_str(&narration.to_string());
        }
        for tag_or_link in &self.tags_and_links {
            ret.push_str(&tag_or_link.pre_ws);
            ret.push_str(&tag_or_link.v.to_string());
        }
        ret.push_str(&self.ws_last);
        ret.push_str(&self.inline_comment);
        ret.push_str(&self.eol);
        ret
    }
}

pub(crate) fn parse_transaction(
    lexer: &mut Lexer<'_>,
    pre_comments: Vec<String>,
    date: Date,
    typ: WithWS<String>,
) -> Option<Directive> {
    let mut t = Transaction {
        pre_comments,
        date,
        typ,
        payee: None,
        narration: None,
        tags_and_links: Default::default(),
        ws_last: "".into(),
        inline_comment: "".into(),
        eol: "".into(),
    };
    let ws1 = match lexer.read_token() {
        Token::Eol(eol) => {
            t.eol = eol.into();
            return Some(Directive::Transaction(t));
        }
        Token::Whitespace(ws) => ws,
        Token::Comment(c) => return parse_transaction_comment(lexer, t, c),
        _ => return None,
    };
    // string or links_and_tags
    let s1 = match lexer.read_token() {
        Token::StringLit(s1) => s1,
        Token::Eol(eol) => {
            t.ws_last = ws1.into();
            t.eol = eol.into();
            return Some(Directive::Transaction(t));
        }
        Token::Comment(c) => {
            t.ws_last = ws1.into();
            return parse_transaction_comment(lexer, t, c);
        }
        Token::Tag(tag) => {
            t.tags_and_links
                .push(WithWS::new(ws1, TagOrLink::Tag(tag.into())));
            return parse_transaction_tags_and_links(lexer, t);
        }
        Token::Link(link) => {
            t.tags_and_links
                .push(WithWS::new(ws1, TagOrLink::Link(link.into())));
            return parse_transaction_tags_and_links(lexer, t);
        }
        _ => return None,
    };
    let ws2 = match lexer.read_token() {
        Token::Whitespace(ws) => ws,
        Token::Eol(eol) => {
            t.narration = Some(WithWS::new(ws1, s1.into()));
            t.eol = eol.into();
            return Some(Directive::Transaction(t));
        }
        Token::Comment(c) => {
            t.narration = Some(WithWS::new(ws1, s1.into()));
            return parse_transaction_comment(lexer, t, c);
        }
        _ => return None,
    };
    let s2 = match lexer.read_token() {
        Token::StringLit(s) => s,
        Token::Eol(eol) => {
            t.narration = Some(WithWS::new(ws1, s1.into()));
            t.ws_last = ws2.into();
            t.eol = eol.into();
            return Some(Directive::Transaction(t));
        }
        Token::Comment(c) => {
            t.narration = Some(WithWS::new(ws1, s1.into()));
            t.ws_last = ws2.into();
            return parse_transaction_comment(lexer, t, c);
        }
        Token::Tag(tag) => {
            t.narration = Some(WithWS::new(ws1, s1.into()));
            t.tags_and_links
                .push(WithWS::new(ws2, TagOrLink::Tag(tag.into())));
            return parse_transaction_tags_and_links(lexer, t);
        }
        Token::Link(link) => {
            t.narration = Some(WithWS::new(ws1, s1.into()));
            t.tags_and_links
                .push(WithWS::new(ws2, TagOrLink::Link(link.into())));
            return parse_transaction_tags_and_links(lexer, t);
        }
        _ => return None,
    };
    t.payee = Some(WithWS::new(ws1, s1.into()));
    t.narration = Some(WithWS::new(ws2, s2.into()));
    let ws3 = match lexer.read_token() {
        Token::Whitespace(ws) => ws,
        Token::Eol(eol) => {
            t.eol = eol.into();
            return Some(Directive::Transaction(t));
        }
        Token::Comment(c) => return parse_transaction_comment(lexer, t, c),
        _ => return None,
    };
    match lexer.read_token() {
        Token::Eol(eol) => {
            t.ws_last = ws3.into();
            t.eol = eol.into();
            Some(Directive::Transaction(t))
        }
        Token::Comment(c) => {
            t.ws_last = ws3.into();
            parse_transaction_comment(lexer, t, c)
        }
        Token::Tag(tag) => {
            t.tags_and_links
                .push(WithWS::new(ws3, TagOrLink::Tag(tag.into())));
            parse_transaction_tags_and_links(lexer, t)
        }
        Token::Link(link) => {
            t.tags_and_links
                .push(WithWS::new(ws3, TagOrLink::Link(link.into())));
            parse_transaction_tags_and_links(lexer, t)
        }
        _ => None,
    }
}
fn parse_transaction_tags_and_links(
    lexer: &mut Lexer<'_>,
    mut t: Transaction,
) -> Option<Directive> {
    loop {
        let ws = match lexer.read_token() {
            Token::Whitespace(ws) => ws,
            Token::Eol(eol) => {
                t.eol = eol.into();
                return Some(Directive::Transaction(t));
            }
            Token::Comment(c) => return parse_transaction_comment(lexer, t, c),
            _ => return None,
        };
        match lexer.read_token() {
            Token::Eol(eol) => {
                t.ws_last = ws.into();
                t.eol = eol.into();
                return Some(Directive::Transaction(t));
            }
            Token::Tag(tag) => {
                t.tags_and_links
                    .push(WithWS::new(ws, TagOrLink::Tag(tag.into())));
            }
            Token::Link(link) => {
                t.tags_and_links
                    .push(WithWS::new(ws, TagOrLink::Link(link.into())));
            }
            Token::Comment(c) => return parse_transaction_comment(lexer, t, c),
            _ => return None,
        }
    }
}

fn parse_transaction_comment(
    lexer: &mut Lexer<'_>,
    mut t: Transaction,
    comment: &str,
) -> Option<Directive> {
    t.inline_comment = comment.into();
    let Token::Eol(eol) = lexer.read_token() else {
        return None;
    };
    t.eol = eol.into();
    Some(Directive::Transaction(t))
}
