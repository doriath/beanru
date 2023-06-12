use std::collections::HashMap;

use beancount_parser_2 as parser;

#[derive(Debug)]
pub struct BeancountFile<D> {
    pub directives: Vec<Directive<D>>,
}

#[derive(Debug)]
pub struct Directive<D> {
    pub date: parser::Date,
    // pub content: DirectiveContent<D>,
    pub metadata: HashMap<String, MetadataValue<D>>,
}

#[derive(Debug)]
pub enum MetadataValue<D> {
    String(String),
    Number(D),
}

impl<D> From<parser::BeancountFile<'_, D>> for BeancountFile<D> {
    fn from(f: parser::BeancountFile<'_, D>) -> Self {
        BeancountFile {
            directives: f.directives.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl<D> From<parser::Directive<'_, D>> for Directive<D> {
    fn from(d: parser::Directive<'_, D>) -> Self {
        Directive {
            date: d.date,
            metadata: HashMap::new(),
        }
    }
}

impl<D> From<parser::MetadataValue<'_, D>> for MetadataValue<D> {
    fn from(v: parser::MetadataValue<'_, D>) -> Self {
        match v {
            parser::MetadataValue::String(x) => MetadataValue::String(x.to_owned()),
            parser::MetadataValue::Number(x) => MetadataValue::Number(x),
            _ => panic!("given metadata value type is not supported yet"),
        }
    }
}

pub fn parse(content: &str) -> anyhow::Result<BeancountFile<rust_decimal::Decimal>> {
    let beancount = match parser::parse::<rust_decimal::Decimal>(&content) {
        Ok(b) => b,
        Err(err) => anyhow::bail!("failed to parse the beancount file: {:?}", err),
    };
    Ok(beancount.into())
}
