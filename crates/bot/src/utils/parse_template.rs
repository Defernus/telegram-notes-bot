pub fn _parse_prompt(template: &str, values: &[(String, String)]) -> String {
    let mut result = template.to_string();

    for (key, value) in values {
        result = result.replace(format!("{{{{{key}}}}}", key = key).as_str(), value);
    }

    result
}
/// Parse a template by replacing the keys with the values.
/// Example:
/// ```rust
/// let template = "Hello, {{name}}!";
/// let values = &[("name", "world")];
/// assert_eq!(parse_template(template, values), "Hello, world!");
/// ```
#[macro_export]
macro_rules! parse_prompt {
    ($template:expr, $($key:ident = $value:expr),* $(,)?) => {
        $crate::_parse_prompt($template, &[$((stringify!($key).to_string(), format!("{}", $value))),*])
    };
}

#[test]
fn test_parse_prompt() {
    let template = "Hello, {{name}}!";
    assert_eq!(parse_prompt!(template, name = "world"), "Hello, world!");
}
