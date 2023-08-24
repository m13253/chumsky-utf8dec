# chumsky-utf8dec
A UTF-8 decoder based on Chumsky parser framework

## Example

```rust
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
```