//! The dictionary service: offline word lookup (FR-NOTE-03).
//!
//! The bundled data set and its parsing live behind the [`Dictionary`] port; the
//! service only normalizes the query and delegates. Normalization lowercases the
//! word and strips surrounding punctuation, so a word selected mid-sentence
//! (with a trailing comma, a curly apostrophe, or quotes) still resolves.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::Definition;
use crate::domain::ports::Dictionary;

/// Looks up word definitions in the bundled offline dictionary.
pub struct DictionaryService {
    dictionary: Arc<dyn Dictionary>,
}

impl DictionaryService {
    pub fn new(dictionary: Arc<dyn Dictionary>) -> Self {
        DictionaryService { dictionary }
    }

    /// The senses of `word`. Returns an empty vector for an unknown or empty
    /// word rather than an error, so the UI can show a plain "no definition".
    pub fn lookup(&self, word: &str) -> Result<Vec<Definition>, DomainError> {
        let normalized = normalize(word);
        if normalized.is_empty() {
            return Ok(Vec::new());
        }
        self.dictionary.lookup(&normalized)
    }
}

/// Lowercase the word and trim non-alphabetic edge characters, keeping inner
/// hyphens and apostrophes so "mother-in-law" and "don't" survive.
fn normalize(word: &str) -> String {
    word.trim()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// A fake dictionary recording the exact query it received.
    struct FakeDictionary {
        entries: HashMap<String, Vec<Definition>>,
        last_query: Mutex<Option<String>>,
    }

    impl FakeDictionary {
        fn new() -> Self {
            let mut entries = HashMap::new();
            entries.insert(
                "book".to_string(),
                vec![Definition {
                    part_of_speech: "noun".to_string(),
                    gloss: "a written work".to_string(),
                    synonyms: vec![],
                    examples: vec![],
                }],
            );
            FakeDictionary {
                entries,
                last_query: Mutex::new(None),
            }
        }
    }

    impl Dictionary for FakeDictionary {
        fn lookup(&self, word: &str) -> Result<Vec<Definition>, DomainError> {
            *self.last_query.lock().unwrap() = Some(word.to_string());
            Ok(self.entries.get(word).cloned().unwrap_or_default())
        }
    }

    fn service() -> (Arc<FakeDictionary>, DictionaryService) {
        let dict = Arc::new(FakeDictionary::new());
        let service = DictionaryService::new(dict.clone());
        (dict, service)
    }

    #[test]
    fn looks_up_a_known_word() {
        let (_dict, service) = service();
        let senses = service.lookup("book").unwrap();
        assert_eq!(senses.len(), 1);
        assert_eq!(senses[0].part_of_speech, "noun");
    }

    #[test]
    fn normalizes_case_and_surrounding_punctuation() {
        let (dict, service) = service();
        let senses = service.lookup("  \u{201c}Book,\u{201d}  ").unwrap();
        assert_eq!(senses.len(), 1);
        // The port saw the normalized form, not the raw selection.
        assert_eq!(dict.last_query.lock().unwrap().as_deref(), Some("book"));
    }

    #[test]
    fn unknown_word_yields_empty_not_error() {
        let (_dict, service) = service();
        assert!(service.lookup("zzzz").unwrap().is_empty());
    }

    #[test]
    fn empty_or_punctuation_only_query_skips_the_port() {
        let (dict, service) = service();
        assert!(service.lookup("   ...  ").unwrap().is_empty());
        // Nothing reached the port, so no query was recorded.
        assert!(dict.last_query.lock().unwrap().is_none());
    }
}
