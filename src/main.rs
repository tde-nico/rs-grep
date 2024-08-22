use std::env;
use std::io;
use std::process;
use std::str::Chars;

#[derive(Clone)]
enum Pattern {
	Literal(char),
	Digit,
	Alphanumeric,
	Group(bool, String),
	End,
	Plus(char),
	Option(char),
	Wildcard,
	Or(Vec<Vec<Pattern>>),
}

fn match_literal(chars: &mut Chars, literal: char) -> bool {
	let c = chars.next();
	c.is_some_and(|c| c == literal)
}

fn match_digit(chars: &mut Chars) -> bool {
	let c = chars.next();
	c.is_some_and(|c| c.is_digit(10))
}

fn match_alphanumeric(chars: &mut Chars) -> bool {
	let c = chars.next();
	c.is_some_and(|c| c.is_alphanumeric())
}

fn match_group(chars: &mut Chars, group: &str) -> bool {
	let c = chars.next();
	c.is_some_and(|c| group.contains(c))
}

fn build_group(it: &mut Chars) -> (bool, String) {
	let mut group = String::new();
	let mut positive = true;
	if it.clone().next().is_some_and(|c| c == '^') {
		positive = false;
		it.next();
	}
	loop {
		let member = it.next();
		if member.is_none() {
			break;
		}
		let member = member.unwrap();
		if member == ']' {
			break;
		}
		group.push(member);
	}
	return (positive, group);
}

fn build_patterrns(pattern: &str) -> Vec<Pattern> {
	let mut it = pattern.chars();
	let mut patterns = Vec::new();
	loop {
		let curr = it.next();
		if curr.is_none() {
			break;
		}
		patterns.push(match curr.unwrap() {
			'\\' => {
				let special = it.next();
				if special.is_none() {
					panic!("Incomplete special character");
				}
				match special.unwrap() {
					'd' => Pattern::Digit,
					'w' => Pattern::Alphanumeric,
					'\\' => Pattern::Literal('\\'),
					_ => panic!("Unknown special character"),
				}
			}
			'[' => {
				let (positive, group) = build_group(&mut it);
				Pattern::Group(positive, group)
			}
			'$' => Pattern::End,
			'.' => Pattern::Wildcard,
			'(' => {
				let mut clone = it.clone();
				let mut len = 0;
				while clone.clone().next().is_some_and(|c| c != ')') {
					clone.next();
					len += 1;
				}
				if clone.clone().next().is_none() {
					panic!("Unmatched '('");
				}
				let sub_pattern = it.as_str()[..len].to_string();
				let subs = sub_pattern.split('|');
				let mut or_patterns = Vec::new();
				for sub in subs {
					or_patterns.push(build_patterrns(sub));
				}
				Pattern::Or(or_patterns)
			}
			l => {
				if it.clone().next().is_some_and(|c| c == '+') {
					it.next();
					Pattern::Plus(l)
				} else if it.clone().next().is_some_and(|c| c == '?') {
					it.next();
					Pattern::Option(l)
				} else {
					Pattern::Literal(l)
				}
			}
		});
	}
	return patterns;
}

fn match_pattern_from(it: &mut Chars, patterns: &Vec<Pattern>) -> bool {
	for p in patterns.iter() {
		match p {
			Pattern::Literal(l) => {
				if !match_literal(it, *l) {
					return false;
				}
			}
			Pattern::Digit => {
				if !match_digit(it) {
					return false;
				}
			}
			Pattern::Alphanumeric => {
				if !match_alphanumeric(it) {
					return false;
				}
			}
			Pattern::Group(positive, group) => {
				if match_group(it, group) != *positive {
					return false;
				}
			}
			Pattern::End => {
				if it.next().is_some() {
					return false;
				}
			}
			Pattern::Plus(l) => {
				if it.clone().next().is_some_and(|c| c == *l) {
					it.next();
					while it.clone().next().is_some_and(|c| c == *l) {
						it.next();
					}
				} else {
					return false;
				}
			}
			Pattern::Option(l) => {
				if it.clone().next().is_some_and(|c| c == *l) {
					it.next();
				}
			}
			Pattern::Wildcard => {
				if it.next().is_none() {
					return false;
				}
			}
			Pattern::Or(sub_patterns) => {
				let mut clone = it.clone();
				let mut matched = false;
				for sub in sub_patterns.iter() {
					if match_pattern_from(&mut clone, &sub.clone()) {
						*it = clone;
						matched = true;
						break ;
					}
					clone = it.clone();
				}
				if matched {
					return true;
				}
			}
		}
	}
	return true;
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
	let input_line = input_line.trim_matches('\n');
	if pattern.chars().nth(0) == Some('^') {
		let patterns = build_patterrns(&pattern[1..]);
		return match_pattern_from(&mut input_line.chars(), &patterns);
	}
	let patterns = build_patterrns(pattern);
	for i in 0..input_line.len() {
		let input = &input_line[i..];
		let mut it = input.chars();
		if match_pattern_from(&mut it, &patterns) {
			return true;
		}
	}

	return false;
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {

	if env::args().nth(1).unwrap() != "-E" {
		process::exit(1);
	}

	let pattern = env::args().nth(2).unwrap();
	let mut input_line = String::new();

	io::stdin().read_line(&mut input_line).unwrap();

	if match_pattern(&input_line, &pattern) {
		process::exit(0)
	} else {
		process::exit(1)
	}
}
