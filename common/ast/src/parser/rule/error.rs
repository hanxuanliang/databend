// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Buffer;
use codespan_reporting::term::Chars;
use codespan_reporting::term::Config;

use crate::parser::rule::util::Input;
use crate::parser::token::TokenKind;

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub errors: Vec<(Input<'a>, ErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    Context(&'static str),
    ExpectToken(TokenKind),
    ExpectText(&'static str),
    Nom(nom::error::ErrorKind),
    ParseIntError(std::num::ParseIntError),
    Other(&'static str),
}

impl<'a> nom::error::ParseError<Input<'a>> for Error<'a> {
    fn from_error_kind(input: Input<'a>, kind: nom::error::ErrorKind) -> Self {
        Error {
            errors: vec![(input, ErrorKind::Nom(kind))],
        }
    }

    fn append(input: Input<'a>, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Nom(kind)));
        other
    }

    fn from_char(_: Input<'a>, _: char) -> Self {
        unreachable!()
    }

    fn or(self, other: Self) -> Self {
        let pos_self = self
            .errors
            .first()
            .and_then(|(input, _)| input.get(0).map(|token| token.span.start))
            .unwrap_or(0);
        let pos_other = other
            .errors
            .first()
            .and_then(|(input, _)| input.get(0).map(|token| token.span.start))
            .unwrap_or(0);

        if pos_other > pos_self {
            other
        } else {
            self
        }
    }
}

impl<'a> nom::error::ContextError<Input<'a>> for Error<'a> {
    fn add_context(input: Input<'a>, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Context(ctx)));
        other
    }
}

impl<'a> Error<'a> {
    pub fn from_error_kind(input: Input<'a>, kind: ErrorKind) -> Self {
        Error {
            errors: vec![(input, kind)],
        }
    }
}

pub fn pretty_print_error<'a>(source: &'a str, error: nom::Err<Error<'a>>) -> String {
    let mut writer = Buffer::no_color();
    let file = SimpleFile::new("SQL", source);
    let config = Config {
        chars: Chars::ascii(),
        before_label_lines: 3,
        ..Default::default()
    };

    let error = match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(_) => unreachable!(),
    };

    let mut lables = Vec::new();
    for (i, (input, kind)) in error.errors.iter().enumerate() {
        let msg = match kind {
            ErrorKind::Context(msg) => format!("while parsing {}", msg),
            ErrorKind::ExpectToken(token) => format!("expected a token of `{:?}`", token),
            ErrorKind::ExpectText(text) => format!("expected token {:?}", text),
            ErrorKind::ParseIntError(err) => format!("nable to parse int: {:?}", err),
            ErrorKind::Other(msg) => msg.to_string(),
            ErrorKind::Nom(_) => continue,
        };

        let span = input[0].span.clone();

        if i == 0 {
            lables.push(Label::primary((), span).with_message(msg));
        } else {
            lables.push(Label::secondary((), span).with_message(msg));
        }
    }

    let diagnostic = Diagnostic::error()
        .with_labels(lables);

    term::emit(&mut writer, &config, &file, &diagnostic).unwrap();

    std::str::from_utf8(&writer.into_inner())
        .unwrap()
        .to_string()
}
