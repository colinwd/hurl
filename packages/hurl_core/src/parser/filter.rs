/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2022 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use crate::ast::{Filter, FilterValue, SourceInfo};
use crate::parser::combinators::choice;
use crate::parser::primitives::{one_or_more_spaces, try_literal};
use crate::parser::query::regex_value;
use crate::parser::{Error, ParseError, ParseResult, Reader};

pub fn filter(reader: &mut Reader) -> ParseResult<'static, Filter> {
    let start = reader.state.pos.clone();
    let value = choice(
        vec![
            count_filter,
            regex_filter,
            escape_url_filter,
            unescape_url_filter,
        ],
        reader,
    )
    .map_err(|e| {
        if e.recoverable {
            Error {
                pos: e.pos,
                recoverable: e.recoverable,
                inner: ParseError::Expecting {
                    value: "filter".to_string(),
                },
            }
        } else {
            e
        }
    })?;
    let end = reader.state.pos.clone();
    let source_info = SourceInfo { start, end };
    Ok(Filter { source_info, value })
}

fn count_filter(reader: &mut Reader) -> ParseResult<'static, FilterValue> {
    try_literal("count", reader)?;
    Ok(FilterValue::Count {})
}

fn regex_filter(reader: &mut Reader) -> ParseResult<'static, FilterValue> {
    try_literal("regex", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let value = regex_value(reader)?;
    Ok(FilterValue::Regex { space0, value })
}

fn escape_url_filter(reader: &mut Reader) -> ParseResult<'static, FilterValue> {
    try_literal("escapeUrl", reader)?;
    Ok(FilterValue::EscapeUrl {})
}

fn unescape_url_filter(reader: &mut Reader) -> ParseResult<'static, FilterValue> {
    try_literal("unescapeUrl", reader)?;
    Ok(FilterValue::UnEscapeUrl {})
}

#[cfg(test)]
mod tests {
    use crate::ast::Pos;
    use crate::parser::ParseError;

    use super::*;

    #[test]
    fn test_count() {
        let mut reader = Reader::init("count");
        assert_eq!(
            filter(&mut reader).unwrap(),
            Filter {
                source_info: SourceInfo::new(1, 1, 1, 6),
                value: FilterValue::Count {},
            }
        );
    }

    #[test]
    fn test_error() {
        let mut reader = Reader::init("xcount");
        let err = filter(&mut reader).err().unwrap();
        assert_eq!(
            err.inner,
            ParseError::Expecting {
                value: "filter".to_string()
            }
        );
        assert_eq!(err.pos, Pos { line: 1, column: 1 });
        assert!(err.recoverable);

        let mut reader = Reader::init("regex 1");
        let err = filter(&mut reader).err().unwrap();
        assert_eq!(
            err.inner,
            ParseError::Expecting {
                value: "\" or /".to_string()
            }
        );
        assert_eq!(err.pos, Pos { line: 1, column: 7 });
        assert!(!err.recoverable);
    }
}
