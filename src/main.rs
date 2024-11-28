use anyhow::Context;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::io::{Lines, Write};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct WhitakerParser;

// pub enum LineKind {
//     Lemma(Vec<String>),
//     Definition(String)
// }

pub struct WhitakerProcess {
    process: Child,
    stdout: BufReader<ChildStdout>,
}

impl WhitakerProcess {
    pub fn new() -> anyhow::Result<Self> {
        let mut process = Command::new("../whitakers-words/bin/words")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir("../whitakers-words")
            .spawn()?;
        dbg!(&process);

        let mut stdout = BufReader::new(process.stdout.take().unwrap());

        stdout.read_until(0, &mut Vec::new())?;

        Ok(Self { process, stdout })
    }

    pub fn lookup_word(&mut self, word: &str) -> anyhow::Result<Vec<String>> {
        self.process
            .stdin
            .as_ref()
            .unwrap()
            .write_all(word.as_bytes())?;
        self.process.stdin.as_ref().unwrap().write_all(b"\n")?;
        let mut result = Vec::new();

        self.stdout.read_until('\n' as u8, &mut Vec::new())?;

        self.stdout.read_until(0, &mut result)?;

        let result = String::from_utf8(result)?;
        let result = format!("{}\n", result.trim());

        let file = WhitakerParser::parse(Rule::file, &result)?;

        let mut lemmata = vec![];

        for line in file {
            match line.as_rule() {
                Rule::entry => {
                    let inner_rule = line.into_inner().next().unwrap();
                    match inner_rule.as_rule() {
                        Rule::possible_lemma => {
                            let form = inner_rule
                                .into_inner()
                                .next()
                                .unwrap()
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str();

                            lemmata.push(form.to_string());
                        }
                        _ => {}
                    }
                }
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }

        //println!("{}", stdout);

        lemmata.sort();
        lemmata.dedup();

        Ok(lemmata)
    }
}

impl Drop for WhitakerProcess {
    fn drop(&mut self) {
        self.process.kill().ok();
    }
}

pub fn lookup_word(word: &str) -> anyhow::Result<Vec<String>> {
    let output = Command::new("../whitakers-words/bin/words")
        .current_dir("../whitakers-words")
        .arg(word.trim())
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let file = WhitakerParser::parse(Rule::file, &stdout)?;

    let mut lemmata = vec![];

    for line in file {
        match line.as_rule() {
            Rule::entry => {
                let inner_rule = line.into_inner().next().unwrap();
                match inner_rule.as_rule() {
                    Rule::possible_lemma => {
                        let form = inner_rule
                            .into_inner()
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap()
                            .as_str();

                        lemmata.push(form.to_string());
                    }
                    _ => {}
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    //println!("{}", stdout);

    lemmata.sort();
    lemmata.dedup();

    dbg!(&lemmata);

    Ok(lemmata)
}

fn main() -> anyhow::Result<()> {
    let mut proc = WhitakerProcess::new()?;
    let filename = "latin_text.txt";
    let content = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;

    let words: Vec<String> = content
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .map(String::from)
        .collect();

    let mut frequency_map: HashMap<String, usize> = HashMap::new();

    for word in words {
        if let Ok(resp) = proc.lookup_word(&word) {
            for entry in resp {
                *frequency_map.entry(entry).or_insert(0) += 1;
            }
        }
    }

    let mut frequency_list: Vec<(&String, &usize)> = frequency_map.iter().collect();
    frequency_list.sort_by(|a, b| b.1.cmp(a.1));

    for (base, count) in frequency_list {
        println!("{:?}: {}", base, count);
    }

    Ok(())
}
