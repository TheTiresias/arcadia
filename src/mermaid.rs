use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::OnceLock;

/// Replace ```` ```mermaid ``` ```` fenced blocks in a markdown string with
/// their inline SVG. Errors include the diagram source for context.
pub fn preprocess(input: &str) -> Result<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?m)^```mermaid\n([\s\S]*?)\n```").unwrap()
    });

    let mut out = String::with_capacity(input.len());
    let mut last_end = 0;

    for cap in re.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let source = cap[1].trim();

        out.push_str(&input[last_end..m.start()]);

        let svg = mermaid_rs_renderer::render(source)
            .map_err(|e| anyhow!("mermaid render failed for diagram {:?}: {}", source, e))?;

        out.push_str(&svg);
        out.push('\n');

        last_end = m.end();
    }

    out.push_str(&input[last_end..]);
    Ok(out)
}
