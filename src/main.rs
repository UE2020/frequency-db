use anyhow::Context;
use pest::Parser;
use pest_derive::Parser;
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdout, Command, Stdio},
    sync::Mutex,
};

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

        let mut stdout = BufReader::new(process.stdout.take().unwrap());

        stdout.skip_until(0)?;

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

        self.stdout.skip_until('\n' as u8)?;

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

fn main() -> anyhow::Result<()> {
    let word_mod_content = "TRIM_OUTPUT                       Y\n\
        HAVE_OUTPUT_FILE                  N\n\
        WRITE_OUTPUT_TO_FILE              N\n\
        DO_UNKNOWNS_ONLY                  N\n\
        WRITE_UNKNOWNS_TO_FILE            N\n\
        IGNORE_UNKNOWN_NAMES              Y\n\
        IGNORE_UNKNOWN_CAPS               Y\n\
        DO_COMPOUNDS                      Y\n\
        DO_FIXES                          Y\n\
        DO_TRICKS                         Y\n\
        DO_DICTIONARY_FORMS               Y\n\
        SHOW_AGE                          N\n\
        SHOW_FREQUENCY                    N\n\
        DO_EXAMPLES                       N\n\
        DO_ONLY_MEANINGS                  N\n\
        DO_STEMS_FOR_UNKNOWN              N\n";

    fs::write("../whitakers-words/WORD.MOD", word_mod_content)?;

    let word_mdv_content = "HAVE_STATISTICS_FILE              N\n\
        WRITE_STATISTICS_FILE             N\n\
        SHOW_DICTIONARY                   N\n\
        SHOW_DICTIONARY_LINE              N\n\
        SHOW_DICTIONARY_CODES             N\n\
        DO_PEARSE_CODES                   N\n\
        DO_ONLY_INITIAL_WORD              N\n\
        FOR_WORD_LIST_CHECK               N\n\
        DO_ONLY_FIXES                     N\n\
        DO_FIXES_ANYWAY                   N\n\
        USE_PREFIXES                      Y\n\
        USE_SUFFIXES                      Y\n\
        USE_TACKONS                       Y\n\
        DO_MEDIEVAL_TRICKS                Y\n\
        DO_SYNCOPE                        N\n\
        DO_TWO_WORDS                      N\n\
        INCLUDE_UNKNOWN_CONTEXT           Y\n\
        NO_MEANINGS                       N\n\
        OMIT_ARCHAIC                      Y\n\
        OMIT_MEDIEVAL                     N\n\
        OMIT_UNCOMMON                     Y\n\
        DO_I_FOR_J                        Y\n\
        DO_U_FOR_V                        N\n\
        PAUSE_IN_SCREEN_OUTPUT            Y\n\
        NO_SCREEN_ACTIVITY                N\n\
        UPDATE_LOCAL_DICTIONARY           N\n\
        UPDATE_MEANINGS                   N\n\
        MINIMIZE_OUTPUT                   Y\n\
        START_FILE_CHARACTER             '@'\n\
        CHANGE_PARAMETERS_CHARACTER      '#'\n\
        CHANGE_DEVELOPER_MODES_CHARACTER '!'\n";

    fs::write("../whitakers-words/WORD.MDV", word_mdv_content)?;

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

    let frequency_map: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
    dbg!(words.len());
    std::thread::scope(|s| {
        for words in words.chunks(words.len() / 32) {
            let words = words.to_vec();
            s.spawn(|| {
                let mut proc = WhitakerProcess::new().unwrap();
                for word in words {
                    let resp = proc.lookup_word(&word);
                    match resp {
                        Ok(resp) => {
                            for entry in resp {
                                *frequency_map.lock().unwrap().entry(entry).or_insert(0) += 1;
                            }
                        }
                        Err(e) => panic!("{word}: {e}"),
                    }
                }
            });
        }
    });

    let frequency_map = frequency_map.into_inner().unwrap();

    let mut frequency_list: Vec<(&String, &usize)> = frequency_map.iter().collect();
    frequency_list.sort_by(|a, b| b.1.cmp(a.1));

    for (base, count) in frequency_list {
        println!("{}\t{}", base, count);
    }

    Ok(())
}
