//! Template representation — loaded from a `.rendertemplate` XML file.
//!
//! A template is an XML document with a root `<template>` element.  Each
//! child `<primitive name="box">…</primitive>` element defines the text
//! fragment emitted for that primitive type.  The text may contain
//! `{placeholder}` tokens that the exporter substitutes at export time.
//!
//! Supported `name` values: `begin`, `end`, `box`, `sphere`, `cylinder`,
//! `mesh`, `line`, `dot`, `grid`, `triangle`, `template`, and any of those
//! with a `::tag` suffix for tagged-primitive overrides (e.g. `box::metal`).
//!
//! # XML format
//! ```xml
//! <template name="My Renderer" defaultExtension="*.sc" runAfter="">
//!   <description>Short description of the template.</description>
//!   <primitive name="begin">
//!     // camera setup …
//!   </primitive>
//!   <primitive name="box">
//!     Box( {matrix} )
//!   </primitive>
//!   <primitive name="end">
//!     // footer …
//!   </primitive>
//! </template>
//! ```

use std::collections::HashMap;
use rustsynth_core::error::Result;

/// One primitive entry in a template — a text fragment with `{placeholder}`
/// tokens.
#[derive(Debug, Clone, Default)]
pub struct TemplatePrimitive {
    pub(crate) text: String,
}

impl TemplatePrimitive {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the fragment contains a specific placeholder.
    pub fn contains(&self, placeholder: &str) -> bool {
        self.text.contains(placeholder)
    }

    /// Return the text with `before` replaced by `after` (all occurrences).
    pub fn substitute(&self, before: &str, after: &str) -> TemplatePrimitive {
        TemplatePrimitive {
            text: self.text.replace(before, after),
        }
    }

    /// Apply a sequence of `(before, after)` substitutions in order,
    /// returning the final string.
    pub fn apply_substitutions<'a>(
        &self,
        substitutions: impl IntoIterator<Item = (&'a str, String)>,
    ) -> String {
        let mut text = self.text.clone();
        for (before, after) in substitutions {
            text = text.replace(before, &after);
        }
        text
    }
}

/// A parsed render template.
#[derive(Debug, Clone, Default)]
pub struct Template {
    /// Template display name (`name` attribute on the root element).
    pub name: String,
    /// Recommended file extension filter (e.g. `"Sunflow scene (*.sc)"`).
    pub default_extension: String,
    /// Optional post-processing shell command (`runAfter` attribute).
    pub run_after: String,
    /// Human-readable description (from `<description>` child).
    pub description: String,
    /// Full original XML text (used for the "edit template" UI).
    pub full_text: String,
    /// Map from primitive name (e.g. `"box"`, `"sphere"`, `"box::metal"`) to
    /// its text fragment.
    pub(crate) primitives: HashMap<String, TemplatePrimitive>,
}

impl Template {
    /// Parse a template from an XML string.
    pub fn from_xml(xml: &str) -> Result<Self> {
        parse_xml(xml)
    }

    /// Check whether a primitive entry exists.
    pub fn has(&self, name: &str) -> bool {
        self.primitives.contains_key(name)
    }

    /// Retrieve a primitive fragment, returning `None` if not present.
    pub fn get(&self, name: &str) -> Option<&TemplatePrimitive> {
        self.primitives.get(name)
    }

    /// Names of all defined primitives (unsorted).
    pub fn primitive_names(&self) -> impl Iterator<Item = &str> {
        self.primitives.keys().map(String::as_str)
    }
}

// ── XML parser (no heavy dependency — minimal hand-rolled parser) ─────────────

/// Parse the template XML.
///
/// We use a simple hand-rolled parser to avoid pulling in `quick-xml` or
/// similar as a hard dependency of this crate.  Templates are small, so
/// performance is not a concern.
fn parse_xml(xml: &str) -> Result<Template> {
    // Extract root element attributes.
    let name = extract_attr(xml, "name").unwrap_or_else(|| "NONAME".to_string());
    let default_extension =
        extract_attr(xml, "defaultExtension").unwrap_or_else(|| "Unknown file type (*.txt)".to_string());
    let run_after = extract_attr(xml, "runAfter").unwrap_or_default();

    // Extract <description> text.
    let description = extract_element_text(xml, "description").unwrap_or_default();

    // Extract all <primitive name="…">…</primitive> entries.
    // Also accept the deprecated tag name <substitution>.
    let mut primitives: HashMap<String, TemplatePrimitive> = HashMap::new();
    for tag in &["primitive", "substitution"] {
        let entries = extract_primitive_elements(xml, tag);
        for (prim_name, prim_text) in entries {
            primitives.insert(prim_name, TemplatePrimitive::new(prim_text));
        }
    }

    Ok(Template {
        name,
        default_extension,
        run_after,
        description,
        full_text: xml.to_string(),
        primitives,
    })
}

/// Naive attribute value extractor for `name="value"` patterns.
fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let needle = format!("{}=\"", attr);
    let start = xml.find(&needle)? + needle.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].to_string())
}

/// Extract the inner text of the first `<tag>…</tag>` element.
fn extract_element_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

/// Extract all `<tag name="…">…</tag>` primitive entries.
fn extract_primitive_elements(xml: &str, tag: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let open_tag = format!("<{} ", tag);
    let close_tag = format!("</{}>", tag);

    let mut rest = xml;
    while let Some(start_pos) = rest.find(&open_tag) {
        // Advance past the opening angle bracket.
        let after_open = &rest[start_pos + 1..]; // skip '<'

        // Find the closing '>' of the opening element tag.
        let tag_end = match after_open.find('>') {
            Some(p) => p,
            None => break,
        };

        let open_elem = &after_open[..tag_end];

        // Extract the `name` attribute from the opening tag text.
        let prim_name = match extract_attr(&format!("<{}", open_elem), "name") {
            Some(n) => n,
            None => {
                rest = &rest[start_pos + open_tag.len()..];
                continue;
            }
        };

        // Optionally extract a `type` attribute (becomes `::type` suffix).
        let type_suffix = match extract_attr(&format!("<{}", open_elem), "type") {
            Some(t) => format!("::{}", t),
            None => String::new(),
        };

        let full_name = format!("{}{}", prim_name, type_suffix);

        // Content starts after the closing '>' of the opening tag.
        let content_start = start_pos + 1 + tag_end + 1;
        let content_rest = &rest[content_start..];

        let close_pos = match content_rest.find(&close_tag) {
            Some(p) => p,
            None => break,
        };

        let content = content_rest[..close_pos].to_string();
        result.push((full_name, content));

        rest = &rest[content_start + close_pos + close_tag.len()..];
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"
<template name="TestTemplate" defaultExtension="*.sc" runAfter="">
  <description>A test template.</description>
  <primitive name="begin">HEADER {width} {height}</primitive>
  <primitive name="box">{matrix} color {r} {g} {b}</primitive>
  <primitive name="sphere">{cx} {cy} {cz} rad {rad}</primitive>
  <primitive name="end">FOOTER</primitive>
</template>
"#;

    #[test]
    fn parse_name() {
        let t = Template::from_xml(SAMPLE_XML).unwrap();
        assert_eq!(t.name, "TestTemplate");
    }

    #[test]
    fn parse_description() {
        let t = Template::from_xml(SAMPLE_XML).unwrap();
        assert_eq!(t.description, "A test template.");
    }

    #[test]
    fn parse_primitives_present() {
        let t = Template::from_xml(SAMPLE_XML).unwrap();
        assert!(t.has("begin"));
        assert!(t.has("box"));
        assert!(t.has("sphere"));
        assert!(t.has("end"));
    }

    #[test]
    fn parse_primitive_text() {
        let t = Template::from_xml(SAMPLE_XML).unwrap();
        assert_eq!(t.get("box").unwrap().text(), "{matrix} color {r} {g} {b}");
    }

    #[test]
    fn substitute_works() {
        let prim = TemplatePrimitive::new("color {r} {g} {b}");
        let out = prim.apply_substitutions([("{r}", "1.0".into()), ("{g}", "0.5".into()), ("{b}", "0.0".into())]);
        assert_eq!(out, "color 1.0 0.5 0.0");
    }
}

