use std::{collections::HashSet, sync::{LazyLock, Mutex}};

use pest::Parser;

use convert_case::{Case, Casing};

#[derive(pest_derive::Parser)]
#[grammar = "ebnf.pest"]
pub struct EbnfParser;

static TOKENS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn parse(input: &str) -> Result<pest::iterators::Pairs<Rule>, pest::error::Error<Rule>> {
    EbnfParser::parse(Rule::Grammar, input)
}

pub fn translate(pairs: pest::iterators::Pairs<Rule>) -> String {
    let mut output = String::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => (),
            Rule::Grammar => for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ProductionOrEmptyLine => for pair in pair.into_inner() {
                        match pair.as_rule() {
                            Rule::Production => translate_production(pair, &mut output),
                            Rule::EmptyLine => (),
                            _ => unreachable!("Unexpected rule in ProductionOrEmptyLine: {:?}", pair.as_rule()),
                        }
                    },
                    Rule::EOI => (),
                    _ => unreachable!("Unexpected rule in Grammar: {:?}", pair.as_rule()),
                }
            }
            _ => unreachable!("Unexpected rule in top level: {:?}", pair.as_rule()),
        }
    }

    for token in TOKENS.lock().unwrap().iter() {
        output.push_str(&format!("{} = {{ ^\"{}\" }}\n", token, token));
    }
    output
}

fn translate_production(pair: pest::iterators::Pair<Rule>, output: &mut String) {
    let mut ident = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Identifier => {
                assert!(ident.replace(translate_identifier(pair)).is_none());
            },
            Rule::RuleOrNote => {
                output.push_str(&ident.take().unwrap());
                output.push_str( " = {\n");
                output.push_str(&translate_rule_or_note(pair));
                output.push_str("}\n");
            },
            _ => unreachable!(),
        }
    }
}

fn translate_identifier(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::IdentifierName => {
                let ident = pair.as_str();
                let ident = ident.replace(" ", "_").replace("/", "_").replace("-", "_");
                let ident = ident.to_case(Case::UpperCamel);
                return ident;
            },
            _ => unreachable!("Unexpected rule in Identifier: {:?}", pair.as_rule()),
        }
    }

    unreachable!("Expected rule not found in Identifier")
}

fn translate_rule_or_note(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Rule => return translate_rule(pair),
            Rule::Note => return translate_note(pair),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn translate_rule(pair: pest::iterators::Pair<Rule>) -> String{
    let mut items = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Concatenation => items.push(format!("({})", translate_concatenation(pair))),
            Rule::BAR => (),
            _ => unreachable!("Unexpected rule in Rule: {:?}", pair.as_rule()),
        }
    }

    items.join(" | ")
}

fn translate_concatenation(pair: pest::iterators::Pair<Rule>) -> String {
    let mut items = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::RepeatableFactor => items.push(translate_repeatable_factor(pair)),
            _ => unreachable!(),
        }
    }

    items.join(" ~ ")
}

fn translate_repeatable_factor(pair: pest::iterators::Pair<Rule>) -> String {
    let mut ellipsis = false;
    let mut op_or_id = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::OperatorOrIdentifier => op_or_id = Some(translate_operator_or_identifier(pair)),
            Rule::ELLIPSIS => ellipsis = true,
            Rule::TerminalOrToken => return translate_terminal_or_token(pair),
            _ => unreachable!(),
        }
    }

    if ellipsis {
        format!("({})+", op_or_id.unwrap())
    } else {
        op_or_id.unwrap()
    }
}

fn translate_operator_or_identifier(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Operator => return translate_operator(pair),
            Rule::Note => return translate_note(pair),
            Rule::Identifier => return translate_identifier(pair),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn translate_terminal_or_token(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Terminal => return translate_terminal(pair),
            Rule::Token => return translate_token(pair),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn translate_terminal(pair: pest::iterators::Pair<Rule>) -> String {
    let mut chars = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::EscapedTerminal => for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::EscapeSequence => for pair in pair.into_inner() {
                        match pair.as_rule() {
                            Rule::EscapedChar => chars.push(pair.as_str().to_string()),
                            _ => unreachable!(),
                        }
                    },
                    Rule::UnescapedTerminal => chars.push(pair.as_str().to_string()),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    chars.join("")
}

fn translate_token(pair: pest::iterators::Pair<Rule>) -> String {
    let token = pair.as_str().to_string();
    TOKENS.lock().unwrap().insert(token.clone());
    token
}

fn translate_operator(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Grouping => return translate_grouping(pair),
            Rule::Option => return translate_option(pair),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn translate_grouping(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Rule => return format!("({})", translate_rule(pair)),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn translate_option(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Rule => return format!("({})?", translate_rule(pair)),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn translate_note(pair: pest::iterators::Pair<Rule>) -> String {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::NoteText => return format!("/* {} */", pair.as_str()),
            Rule::NOTE => (),
            _ => unreachable!("Unexpected rule in Note: {:?}", pair.as_rule()),
        }
    }

    unreachable!()
}

