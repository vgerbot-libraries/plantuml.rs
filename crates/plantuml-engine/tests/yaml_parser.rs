//! Minimal YAML parser for vega test headers.
//!
//! Only supports the simple key-value format used in vega .puml headers:
//! ```
//! ---
//! output: preproc
//! ---
//! ```

/// Parsed YAML header from a vega .puml file.
#[derive(Debug, Clone, Default)]
pub struct VegaYaml {
    /// The `output` key value (e.g. "preproc", "svg").
    pub output: Option<String>,
    /// The `allow-failure` key value.
    pub allow_failure: bool,
    /// The `expected-exception` key value.
    pub expected_exception: Option<String>,
    /// The `expected-error-line` key value.
    pub expected_error_line: Option<String>,
    /// The `expected-error-message` key value.
    pub expected_error_message: Option<String>,
    /// The `expected-image-count` key value.
    pub expected_image_count: Option<String>,
    /// The `expected-description` key value.
    pub expected_description: Option<String>,
    /// The `verbose` key value.
    pub verbose: bool,
    /// The `decimal` key value.
    pub decimal: Option<String>,
    /// The `tag` key value.
    pub tag: Option<String>,
}

impl VegaYaml {
    /// Parses YAML lines (between `---` delimiters) into a `VegaYaml`.
    pub fn parse(yaml_lines: &[String]) -> Self {
        let mut result = Self::default();
        for line in yaml_lines {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "output" => result.output = Some(value.to_string()),
                    "allow-failure" => result.allow_failure = value == "true",
                    "expected-exception" => result.expected_exception = Some(value.to_string()),
                    "expected-error-line" => result.expected_error_line = Some(value.to_string()),
                    "expected-error-message" => {
                        result.expected_error_message = Some(value.to_string());
                    }
                    "expected-image-count" => result.expected_image_count = Some(value.to_string()),
                    "expected-description" => result.expected_description = Some(value.to_string()),
                    "verbose" => result.verbose = value == "true",
                    "decimal" => result.decimal = Some(value.to_string()),
                    "tag" => result.tag = Some(value.to_string()),
                    _ => {}
                }
            }
        }
        result
    }
}
