use rand::Rng;

/// Expand spintax syntax `{Option A|Option B}` in a string.
/// Nested spintax is supported.
pub fn expand_spintax(input: &str) -> String {
    let mut rng = rand::thread_rng();
    expand_inner(input, &mut rng)
}

fn expand_inner(input: &str, rng: &mut impl Rng) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '{' => {
                // Find matching closing brace, respecting nesting
                let mut depth = 1;
                let mut j = i + 1;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        j += 1;
                    }
                }

                if j < chars.len() {
                    // Extract inner content (between braces)
                    let inner: String = chars[i + 1..j].iter().collect();
                    // Split on top-level `|` (not inside nested braces)
                    let options = split_top_level(&inner);
                    if options.len() > 1 {
                        let idx = rng.gen_range(0..options.len());
                        result.push_str(&expand_inner(&options[idx], rng));
                    } else {
                        // No pipe found — treat as literal text
                        result.push('{');
                        result.push_str(&expand_inner(&inner, rng));
                        result.push('}');
                    }
                    i = j + 1;
                } else {
                    // Unclosed brace — literal
                    result.push('{');
                    i += 1;
                }
            }
            c => {
                result.push(c);
                i += 1;
            }
        }
    }

    result
}

/// Split a string on top-level `|` characters (ignoring those inside `{}`).
fn split_top_level(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth: usize = 0;

    for c in input.chars() {
        match c {
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            '|' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() || !parts.is_empty() {
        parts.push(current);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_spintax() {
        let result = expand_spintax("{Hello|Hi} there");
        assert!(result == "Hello there" || result == "Hi there");
    }

    #[test]
    fn test_no_spintax() {
        assert_eq!(expand_spintax("plain text"), "plain text");
    }

    #[test]
    fn test_nested_spintax() {
        let result = expand_spintax("{A {B|C}|D}");
        assert!(result == "A B" || result == "A C" || result == "D");
    }

    #[test]
    fn test_multiple_spintax() {
        let result = expand_spintax("{a|b} {c|d}");
        assert!(
            result == "a c" || result == "a d" || result == "b c" || result == "b d"
        );
    }
}