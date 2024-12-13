pub fn remove_braces(text: &str) -> String {
    text.replace(&['{', '}'][..], "")
}

fn wrap_word_with_braces(word: &str) -> String {
    word.strip_suffix(':').map_or_else(
        || format!("{{{}}}", word),
        |stripped| format!("{{{}}}:", stripped),
    )
}

pub fn format_title(text: &str) -> String {
    remove_braces(text)
        .split_whitespace()
        .map(|word| {
            if word.chars().any(|c| c.is_uppercase()) {
                wrap_word_with_braces(word)
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("foo", "foo" ; "default")]
    #[test_case("{foo}", "foo" ; "simple")]
    #[test_case("{foo} {} {bar}}", "foo  bar" ; "braces complex")]
    fn test_remove_braces(input: &str, expected: &str) {
        assert_eq!(remove_braces(input), expected)
    }

    #[test_case("foo", "{foo}" ; "default")]
    #[test_case("foo:", "{foo}:" ; "exclude colon")]
    fn test_wrap_word_with_braces(input: &str, expected: &str) {
        assert_eq!(wrap_word_with_braces(input), expected)
    }

    #[test_case("foo", "foo" ; "default")]
    #[test_case("{foo}", "foo" ; "simple")]
    #[test_case("FOO:", "{FOO}:" ; "exclude colon")]
    #[test_case("{FOO: A Framework for BAR}", "{FOO}: {A} {Framework} for {BAR}" ; "multiple")]
    fn test_format_title(input: &str, expected: &str) {
        assert_eq!(format_title(input), expected)
    }
}
