//! The UCI configuration file format (chapter 6 §3): parsing a file into sections, rewriting
//! only the sections Enkrnett owns, and serialising. Foreign sections are kept byte for byte in
//! their order (ENK-CFG-001, ENK-CFG-015). No I/O.
//!
//! Format: `config <type> ['<name>']`, then indented `option <key> '<value>'` and
//! `list <key> '<value>'` lines. Values are single-quoted; a single quote inside a value is
//! written as `'\''`, as the `uci` tool writes it. Comments and blank lines outside sections
//! are kept; comments inside a section are dropped when that section is rewritten.

/// One `config` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub kind: String,
    /// `None` for an anonymous section (`config rule`).
    pub name: Option<String>,
    /// `option` and `list` entries in order; a list key appears once with several values.
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Option(String, String),
    List(String, Vec<String>),
}

impl Section {
    pub fn named(kind: &str, name: &str) -> Section {
        Section {
            kind: kind.to_string(),
            name: Some(name.to_string()),
            entries: Vec::new(),
        }
    }

    pub fn option(mut self, key: &str, value: impl Into<String>) -> Section {
        self.entries
            .push(Entry::Option(key.to_string(), value.into()));
        self
    }

    pub fn list(mut self, key: &str, values: &[&str]) -> Section {
        self.entries.push(Entry::List(
            key.to_string(),
            values.iter().map(|v| v.to_string()).collect(),
        ));
        self
    }

    /// The same section whatever the order of its entries: a section the daemon rendered and
    /// a section regenerated from it may list the same options in another order.
    pub fn equivalent(&self, other: &Section) -> bool {
        if self.kind != other.kind
            || self.name != other.name
            || self.entries.len() != other.entries.len()
        {
            return false;
        }
        let mut a: Vec<String> = self.entries.iter().map(|e| format!("{e:?}")).collect();
        let mut b: Vec<String> = other.entries.iter().map(|e| format!("{e:?}")).collect();
        a.sort();
        b.sort();
        a == b
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.iter().find_map(|e| match e {
            Entry::Option(k, v) if k == key => Some(v.as_str()),
            _ => None,
        })
    }
}

/// A whole file: raw lines, with every section's line range known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    lines: Vec<String>,
    /// (first line index, one past last line index, parsed section)
    spans: Vec<(usize, usize, Section)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: &'static str,
}

fn unquote(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if let Some(inner) = raw.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        Some(inner.replace("'\\''", "'"))
    } else if let Some(inner) = raw.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        Some(inner.replace("\\\"", "\""))
    } else if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Splits a line into its first word and the rest.
fn split_word(line: &str) -> (&str, &str) {
    let line = line.trim();
    match line.find(char::is_whitespace) {
        Some(i) => (&line[..i], line[i..].trim()),
        None => (line, ""),
    }
}

impl File {
    pub fn parse(text: &str) -> Result<File, ParseError> {
        let lines: Vec<String> = text.lines().map(str::to_string).collect();
        let mut spans = Vec::new();
        let mut current: Option<(usize, Section)> = None;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (word, rest) = split_word(trimmed);
            match word {
                "config" => {
                    if let Some((start, section)) = current.take() {
                        spans.push((start, i, section));
                    }
                    let (kind, name) = split_word(rest);
                    if kind.is_empty() {
                        return Err(ParseError {
                            line: i + 1,
                            message: "config without a type",
                        });
                    }
                    current = Some((
                        i,
                        Section {
                            kind: kind.to_string(),
                            name: unquote(name),
                            entries: Vec::new(),
                        },
                    ));
                }
                "option" | "list" => {
                    let Some((_, section)) = current.as_mut() else {
                        return Err(ParseError {
                            line: i + 1,
                            message: "option outside a section",
                        });
                    };
                    let (key, value) = split_word(rest);
                    if key.is_empty() {
                        return Err(ParseError {
                            line: i + 1,
                            message: "option without a key",
                        });
                    }
                    let value = unquote(value).unwrap_or_default();
                    if word == "option" {
                        section.entries.push(Entry::Option(key.to_string(), value));
                    } else if let Some(Entry::List(k, values)) = section
                        .entries
                        .iter_mut()
                        .find(|e| matches!(e, Entry::List(k, _) if k == key))
                    {
                        debug_assert_eq!(k, key);
                        values.push(value);
                    } else {
                        section
                            .entries
                            .push(Entry::List(key.to_string(), vec![value]));
                    }
                }
                "package" => {}
                _ => {
                    return Err(ParseError {
                        line: i + 1,
                        message: "unknown keyword",
                    });
                }
            }
        }
        if let Some((start, section)) = current {
            spans.push((start, lines.len(), section));
        }
        Ok(File { lines, spans })
    }

    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.spans.iter().map(|(_, _, s)| s)
    }

    pub fn named_section(&self, name: &str) -> Option<&Section> {
        self.sections().find(|s| s.name.as_deref() == Some(name))
    }

    /// Serialises one section as `uci` writes it.
    pub fn render_section(section: &Section) -> String {
        let mut out = String::new();
        match &section.name {
            Some(n) => out.push_str(&format!("config {} {}\n", section.kind, quote(n))),
            None => out.push_str(&format!("config {}\n", section.kind)),
        }
        for e in &section.entries {
            match e {
                Entry::Option(k, v) => out.push_str(&format!("\toption {k} {}\n", quote(v))),
                Entry::List(k, values) => {
                    for v in values {
                        out.push_str(&format!("\tlist {k} {}\n", quote(v)));
                    }
                }
            }
        }
        out
    }

    /// Returns the file with every section whose name has `prefix` removed and `owned` appended
    /// in the given order, foreign lines untouched (ENK-CFG-001). Anonymous sections are never
    /// removed.
    pub fn replace_owned(&self, prefix: &str, owned: &[Section]) -> String {
        self.rewrite(prefix, owned, &[])
    }

    /// As [`File::replace_owned`], and additionally replaces, at their own position, the sections
    /// whose name equals one in `in_place` (the `wifi-device` sections, whose names the platform
    /// chose; ENK-CFG-003).
    pub fn rewrite(&self, prefix: &str, owned: &[Section], in_place: &[Section]) -> String {
        let mut out = String::new();
        let mut i = 0;
        for (start, end, section) in &self.spans {
            if i < *start {
                for l in &self.lines[i..*start] {
                    out.push_str(l);
                    out.push('\n');
                }
            }
            let is_owned = section
                .name
                .as_deref()
                .is_some_and(|n| n.starts_with(prefix));
            let replacement = section
                .name
                .as_ref()
                .and_then(|n| in_place.iter().find(|r| r.name.as_ref() == Some(n)));
            if let Some(r) = replacement {
                out.push_str(&File::render_section(r));
            } else if !is_owned {
                for l in &self.lines[*start..*end] {
                    out.push_str(l);
                    out.push('\n');
                }
            }
            i = *end;
        }
        for l in &self.lines[i..] {
            out.push_str(l);
            out.push('\n');
        }
        let trimmed = out.trim_end_matches('\n');
        let mut result = if trimmed.is_empty() {
            String::new()
        } else {
            format!("{trimmed}\n")
        };
        for s in owned {
            result.push('\n');
            result.push_str(&File::render_section(s));
        }
        result
    }

    /// Whether the file's owned sections equal `owned` exactly (ENK-CFG-014, drift).
    pub fn owned_equal(&self, prefix: &str, owned: &[Section]) -> bool {
        let mine: Vec<&Section> = self
            .sections()
            .filter(|s| s.name.as_deref().is_some_and(|n| n.starts_with(prefix)))
            .collect();
        mine.len() == owned.len() && mine.iter().zip(owned).all(|(a, b)| a.equivalent(b))
    }
}

#[cfg(test)]
mod tests;
