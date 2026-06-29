//! The offline dictionary adapter over a bundled WordNet JSON data set.
//!
//! WordNet ships as synsets (sets of synonymous words sharing one gloss). This
//! adapter inverts that into a word-to-senses index so a lookup is a single hash
//! probe. The data set is large, so the index is built lazily on first use and
//! cached for the process lifetime, keeping it off the launch path (NFR-P-01).
//! All parsing lives here; the domain core sees only [`Definition`] values.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::model::Definition;
use crate::domain::ports::Dictionary;

/// A WordNet-backed dictionary. The index is built once, the first time a word
/// is looked up, from the JSON file at `path`.
pub struct WordNetDictionary {
    path: PathBuf,
    index: OnceLock<HashMap<String, Vec<Definition>>>,
}

impl WordNetDictionary {
    pub fn new(path: PathBuf) -> Self {
        WordNetDictionary {
            path,
            index: OnceLock::new(),
        }
    }

    /// The word index, built on first access. A parse failure is folded into an
    /// empty index so a missing or corrupt data set degrades to "no definition"
    /// rather than failing every lookup.
    fn index(&self) -> &HashMap<String, Vec<Definition>> {
        self.index.get_or_init(|| match self.build_index() {
            Ok(index) => index,
            Err(e) => {
                eprintln!("dictionary: failed to load {}: {e}", self.path.display());
                HashMap::new()
            }
        })
    }

    fn build_index(&self) -> Result<HashMap<String, Vec<Definition>>, DomainError> {
        let bytes = std::fs::read(&self.path)
            .map_err(|e| DomainError::Dictionary(format!("read {}: {e}", self.path.display())))?;
        let file: WordNetFile = serde_json::from_slice(&bytes)
            .map_err(|e| DomainError::Dictionary(format!("parse: {e}")))?;

        let mut index: HashMap<String, Vec<Definition>> = HashMap::new();
        for synset in file.synset.into_values() {
            let (gloss, examples) = split_gloss(&synset.gloss);
            for (i, word) in synset.word.iter().enumerate() {
                let key = word.to_lowercase();
                // The headword's synonyms are the other words in the synset.
                let synonyms = synset
                    .word
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, w)| w.clone())
                    .collect();
                index.entry(key).or_default().push(Definition {
                    part_of_speech: part_of_speech(&synset.pos).to_string(),
                    gloss: gloss.clone(),
                    synonyms,
                    examples: examples.clone(),
                });
            }
        }
        Ok(index)
    }
}

impl Dictionary for WordNetDictionary {
    fn lookup(&self, word: &str) -> Result<Vec<Definition>, DomainError> {
        Ok(self.index().get(word).cloned().unwrap_or_default())
    }
}

/// The slice of the WordNet JSON we parse: a map of synset id to its content.
/// Every other field in the file (pointers, frames, offsets) is ignored.
#[derive(Deserialize)]
struct WordNetFile {
    synset: HashMap<String, RawSynset>,
}

#[derive(Deserialize)]
struct RawSynset {
    pos: String,
    word: Vec<String>,
    gloss: String,
}

/// Expand a WordNet part-of-speech code to a readable label.
fn part_of_speech(code: &str) -> &'static str {
    match code {
        "n" => "noun",
        "v" => "verb",
        // `a` is an adjective, `s` an adjective satellite; both read as adjective.
        "a" | "s" => "adjective",
        "r" => "adverb",
        _ => "other",
    }
}

/// Split a WordNet gloss into its definition and example sentences. A gloss is
/// the definition followed by zero or more quoted examples, separated by
/// semicolons, e.g. `a written work; "she read a book"`. Quoted clauses become
/// examples; the rest is the definition.
fn split_gloss(raw: &str) -> (String, Vec<String>) {
    let mut definition_parts = Vec::new();
    let mut examples = Vec::new();
    for part in raw.split(';') {
        let part = part.trim();
        if part.starts_with('"') {
            examples.push(part.trim_matches('"').trim().to_string());
        } else if !part.is_empty() {
            definition_parts.push(part);
        }
    }
    (definition_parts.join("; "), examples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A self-deleting temp JSON file holding the given content.
    struct Fixture {
        path: PathBuf,
    }

    impl Fixture {
        fn new(json: &str) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("prose_dict_test_{}_{n}.json", std::process::id()));
            std::fs::write(&path, json).unwrap();
            Fixture { path }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn write_fixture(json: &str) -> Fixture {
        Fixture::new(json)
    }

    #[test]
    fn splits_definition_from_quoted_examples() {
        let (def, ex) = split_gloss("a written work; \"she read a book\"; \"buy a book\"");
        assert_eq!(def, "a written work");
        assert_eq!(ex, vec!["she read a book", "buy a book"]);
    }

    #[test]
    fn maps_pos_codes_to_labels() {
        assert_eq!(part_of_speech("n"), "noun");
        assert_eq!(part_of_speech("s"), "adjective");
        assert_eq!(part_of_speech("r"), "adverb");
    }

    #[test]
    fn inverts_synsets_into_a_word_index_with_synonyms() {
        let json = r#"{
            "synset": {
                "n1": { "pos": "n", "word": ["book", "volume"], "gloss": "a written work; \"read a book\"" }
            }
        }"#;
        let file = write_fixture(json);
        let dict = WordNetDictionary::new(file.path.clone());

        let senses = dict.lookup("book").unwrap();
        assert_eq!(senses.len(), 1);
        assert_eq!(senses[0].part_of_speech, "noun");
        assert_eq!(senses[0].gloss, "a written work");
        assert_eq!(senses[0].synonyms, vec!["volume"]);
        assert_eq!(senses[0].examples, vec!["read a book"]);

        // The synonym is indexed too, pointing back with the headword as its synonym.
        assert_eq!(dict.lookup("volume").unwrap()[0].synonyms, vec!["book"]);
    }

    #[test]
    fn unknown_word_is_empty() {
        let file = write_fixture(r#"{ "synset": {} }"#);
        let dict = WordNetDictionary::new(file.path.clone());
        assert!(dict.lookup("missing").unwrap().is_empty());
    }

    #[test]
    fn a_missing_data_file_degrades_to_empty() {
        let dict = WordNetDictionary::new(PathBuf::from("/no/such/wordnet.json"));
        assert!(dict.lookup("book").unwrap().is_empty());
    }
}
