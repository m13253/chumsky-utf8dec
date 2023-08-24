use chumsky::extra::ParserExtra;
use chumsky::input::ValueInput;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::span::Span;

/// Returns a Chumsky parser that parses a sequence of bytes (e.g. `&[u8]`),
/// into a collection of `(char, Span)`.
pub fn decoder<
    'a,
    I: Input<'a, Token = u8, Span = S> + ValueInput<'a>,
    S: Span,
    E: ParserExtra<'a, I>,
>() -> impl IterParser<'a, I, (char, S), E>
where
    <E as ParserExtra<'a, I>>::Error: LabelError<'a, I, &'a str>,
{
    codepoint().map_with_span(|c, span| (c, span)).repeated()
}

/// Returns a Chumsky parser that parses a single codepoint from a sequence of
/// bytes (e.g., `&[u8]`), into a single Unicode codepoint of `char` type.
pub fn codepoint<
    'a,
    I: Input<'a, Token = u8, Span = S> + ValueInput<'a>,
    S: Span,
    E: ParserExtra<'a, I>,
>() -> impl Parser<'a, I, char, E>
where
    <E as ParserExtra<'a, I>>::Error: LabelError<'a, I, &'a str>,
{
    let start_00_7f = one_of(0x00..=0x7F).labelled("a UTF-8 start byte");
    let start_c2_df = one_of(0xC2..=0xDF).labelled("a UTF-8 start byte");
    let start_e0_e0 = one_of(0xE0..=0xE0).labelled("a UTF-8 start byte");
    let start_e1_ec = one_of(0xE1..=0xEC).labelled("a UTF-8 start byte");
    let start_ed_ed = one_of(0xED..=0xED).labelled("a UTF-8 start byte");
    let start_ee_ef = one_of(0xEE..=0xEF).labelled("a UTF-8 start byte");
    let start_f0_f0 = one_of(0xF0..=0xF0).labelled("a UTF-8 start byte");
    let start_f1_f3 = one_of(0xF1..=0xF3).labelled("a UTF-8 start byte");
    let start_f4_f4 = one_of(0xF4..=0xF4).labelled("a UTF-8 start byte");

    let continuation = one_of(0x80..=0xBF).labelled("a UTF-8 continuation byte");
    let continue_e0 = one_of(0xA0..=0xBF).labelled("0xA0 ..= 0xBF");
    let continue_ed = one_of(0x80..=0x9F).labelled("0x80 ..= 0x9F");
    let continue_f0 = one_of(0x90..=0xBF).labelled("0x90 ..= 0xBF");
    let continue_f4 = one_of(0x80..=0x8F).labelled("0x80 ..= 0x8F");

    let transform_1 = |a| a as u32;
    let transform_2 = |(a, b)| (a as u32 & 0x1F) << 6 | (b as u32 & 0x3F);
    let transform_3 =
        |((a, b), c)| (a as u32 & 0x0F) << 12 | (b as u32 & 0x3F) << 6 | (c as u32 & 0x3F);
    let transform_4 = |(((a, b), c), d)| {
        (a as u32 & 0x07) << 18
            | (b as u32 & 0x3F) << 12
            | (c as u32 & 0x3F) << 6
            | (d as u32 & 0x3F)
    };

    choice((
        start_00_7f.map(transform_1),
        start_c2_df.then(continuation.clone()).map(transform_2),
        start_e0_e0
            .then(continue_e0)
            .then(continuation.clone())
            .map(transform_3),
        start_e1_ec
            .then(continuation.clone())
            .then(continuation.clone())
            .map(transform_3),
        start_ed_ed
            .then(continue_ed)
            .then(continuation.clone())
            .map(transform_3),
        start_ee_ef
            .then(continuation.clone())
            .then(continuation.clone())
            .map(transform_3),
        start_f0_f0
            .then(continue_f0)
            .then(continuation.clone())
            .then(continuation.clone())
            .map(transform_4),
        start_f1_f3
            .then(continuation.clone())
            .then(continuation.clone())
            .then(continuation.clone())
            .map(transform_4),
        start_f4_f4
            .then(continue_f4)
            .then(continuation.clone())
            .then(continuation)
            .map(transform_4),
    ))
    .map(|c| char::from_u32(c).unwrap())
    .labelled("a Unicode codepoint")
}

#[test]
fn demo_success() {
    let (output, errors) = decoder::<_, _, extra::Err<Rich<_, _>>>()
        .collect::<Vec<_>>()
        .parse("👨‍👩‍👧‍👦".as_bytes().with_context("input1.txt"))
        .into_output_errors();
    assert_eq!(
        output,
        Some(vec![
            ('\u{1F468}', ("input1.txt", SimpleSpan::new(0, 4))),
            ('\u{200D}', ("input1.txt", SimpleSpan::new(4, 7))),
            ('\u{1F469}', ("input1.txt", SimpleSpan::new(7, 11))),
            ('\u{200D}', ("input1.txt", SimpleSpan::new(11, 14))),
            ('\u{1F467}', ("input1.txt", SimpleSpan::new(14, 18))),
            ('\u{200D}', ("input1.txt", SimpleSpan::new(18, 21))),
            ('\u{1F466}', ("input1.txt", SimpleSpan::new(21, 25)))
        ])
    );
    assert_eq!(errors, vec![]);
}

#[test]
fn demo_failure() {
    use chumsky::error::{RichPattern, RichReason};
    use chumsky::util::Maybe;

    let (output, errors) = decoder::<_, _, extra::Err<Rich<_, _>>>()
        .collect::<Vec<_>>()
        .parse(b"\xED\xA0\x80".with_context("input2.txt"))
        .into_output_errors();
    assert_eq!(output, None);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].reason(),
        &RichReason::ExpectedFound {
            expected: vec![RichPattern::Label("0x80 ..= 0x9F")],
            found: Some(Maybe::Val(0xA0)),
        }
    );
    assert_eq!(errors[0].span(), &("input2.txt", SimpleSpan::new(1, 2)));
}
