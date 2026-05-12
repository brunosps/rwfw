use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
pub struct ViteManifest {
    #[serde(flatten)]
    pub entries: HashMap<String, ManifestEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestEntry {
    pub file: String,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub imports: Vec<String>,
    #[serde(rename = "isEntry", default)]
    pub is_entry: bool,
}

impl ViteManifest {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: ViteManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    pub fn entry_script(&self, entry: &str) -> Option<&str> {
        self.entries.get(entry).map(|e| e.file.as_str())
    }

    pub fn first_entry_key(&self) -> Option<&str> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.is_entry)
            .map(|(key, _)| key.as_str())
            .or_else(|| self.entries.keys().next().map(String::as_str))
    }

    pub fn entry_css(&self, entry: &str) -> Vec<&str> {
        let mut css = Vec::new();
        let mut seen_entries = HashSet::new();
        let mut seen_css = HashSet::new();
        self.collect_entry_css(entry, &mut seen_entries, &mut seen_css, &mut css);
        css
    }

    fn collect_entry_css<'a>(
        &'a self,
        entry: &str,
        seen_entries: &mut HashSet<String>,
        seen_css: &mut HashSet<&'a str>,
        css: &mut Vec<&'a str>,
    ) {
        if !seen_entries.insert(entry.to_string()) {
            return;
        }

        let Some(manifest_entry) = self.entries.get(entry) else {
            return;
        };

        for import in &manifest_entry.imports {
            self.collect_entry_css(import, seen_entries, seen_css, css);
        }

        for file in &manifest_entry.css {
            if seen_css.insert(file.as_str()) {
                css.push(file.as_str());
            }
        }
    }
}
