use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct WhitakerParser;

pub enum LineKind {
    Lemma(Vec<String>),
    Definition(String)
}

pub fn lookup_word(word: &str) -> anyhow::Result<()> {
    let output = Command::new("../whitakers-words/bin/words")
        .current_dir("../whitakers-words")
        .arg(word.trim())
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let file = WhitakerParser::parse(Rule::file, &stdout)?;

    for line in file {
        match line.as_rule() {
            Rule::entry => {
                let inner_rule = line.into_inner().next().unwrap();
                match inner_rule.as_rule() {
                    Rule::possible_lemma => {

                    }
                    _ => {}
                }
                dbg!(inner_rule.as_rule());
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("{}", stdout);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let unparsed_file = fs::read_to_string("test.txt")?;

    lookup_word("factum")?;
    // let file = WhitakerParser::parse(Rule::file, &unparsed_file)?;
    //
    // println!("{:#?}", file);

    Ok(())
}
