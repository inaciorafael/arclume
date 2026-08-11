use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use quick_xml::Reader;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};

const SOURCE: &str = "Dicionário Aberto · CC BY-SA 2.5 PT";
const MAX_CACHE_ENTRIES: usize = 256;
const CACHE_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryEntry {
    pub word: String,
    pub definitions: Vec<String>,
    pub source: String,
    pub cached: bool,
}

#[derive(Clone, Deserialize, Serialize)]
struct CachedEntry {
    entry: DictionaryEntry,
    cached_at: i64,
}

#[derive(Deserialize)]
struct ApiEntry {
    word: String,
    xml: String,
}

pub struct DictionaryProvider {
    client: reqwest::Client,
    cache_path: PathBuf,
    cache: Mutex<HashMap<String, CachedEntry>>,
}

impl DictionaryProvider {
    pub fn open(database_path: &Path) -> Result<Self, String> {
        let cache_path = database_path
            .parent()
            .ok_or("dictionary cache directory is unavailable")?
            .join("dictionary-cache.json");
        let cache = fs::read(&cache_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(4))
            .user_agent("Arclume/0.2 dictionary lookup")
            .build()
            .map_err(|error| format!("failed to initialize dictionary client: {error}"))?;
        Ok(Self {
            client,
            cache_path,
            cache: Mutex::new(cache),
        })
    }

    pub async fn lookup(&self, word: &str) -> Result<DictionaryEntry, String> {
        let word = validate_word(word)?;
        if let Some(entry) = self.cached(&word) {
            return Ok(entry);
        }
        let mut url = reqwest::Url::parse("https://api.dicionario-aberto.net/word/")
            .map_err(|error| error.to_string())?;
        url.path_segments_mut()
            .map_err(|_| "dictionary endpoint cannot accept path segments")?
            .push(&word);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| format!("dictionary is unavailable: {error}"))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("nenhuma definição encontrada para “{word}”"));
        }
        let response = response
            .error_for_status()
            .map_err(|error| format!("dictionary request failed: {error}"))?;
        if response
            .content_length()
            .is_some_and(|length| length > 512 * 1024)
        {
            return Err("dictionary response exceeded the 512 KB limit".into());
        }
        let api_entries: Vec<ApiEntry> = response
            .json()
            .await
            .map_err(|error| format!("dictionary returned invalid data: {error}"))?;
        let mut definitions = api_entries
            .iter()
            .flat_map(|entry| definitions_from_xml(&entry.xml).unwrap_or_default())
            .filter(|definition| !definition.is_empty())
            .collect::<Vec<_>>();
        definitions.dedup();
        definitions.truncate(6);
        let display_word = api_entries
            .first()
            .map_or_else(|| word.clone(), |entry| entry.word.clone());
        if definitions.is_empty() {
            return Err(format!("nenhuma definição encontrada para “{word}”"));
        }
        let entry = DictionaryEntry {
            word: display_word,
            definitions,
            source: SOURCE.into(),
            cached: false,
        };
        self.store(word, entry.clone());
        Ok(entry)
    }

    fn cached(&self, word: &str) -> Option<DictionaryEntry> {
        let cache = self.cache.lock().ok()?;
        let cached = cache.get(word)?;
        (now_seconds() - cached.cached_at <= CACHE_TTL_SECONDS).then(|| {
            let mut entry = cached.entry.clone();
            entry.cached = true;
            entry
        })
    }

    fn store(&self, word: String, entry: DictionaryEntry) {
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        cache.insert(
            word,
            CachedEntry {
                entry,
                cached_at: now_seconds(),
            },
        );
        if cache.len() > MAX_CACHE_ENTRIES
            && let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, cached)| cached.cached_at)
                .map(|(word, _)| word.clone())
        {
            cache.remove(&oldest);
        }
        if let Ok(bytes) = serde_json::to_vec(&*cache) {
            let _ = fs::write(&self.cache_path, bytes);
        }
    }
}

pub fn query_word(query: &str) -> Option<String> {
    let trimmed = query.trim();
    let lowercase = trimmed.to_lowercase();
    ["definir ", "significado de ", "dicionário ", "dicionario "]
        .into_iter()
        .find_map(|prefix| lowercase.strip_prefix(prefix))
        .and_then(|word| validate_word(word).ok())
}

fn validate_word(word: &str) -> Result<String, String> {
    let word = word.trim().to_lowercase();
    if word.is_empty()
        || word.chars().count() > 64
        || !word
            .chars()
            .all(|character| character.is_alphabetic() || character == '-' || character == ' ')
    {
        return Err("use uma palavra portuguesa com até 64 caracteres".into());
    }
    Ok(word)
}

fn definitions_from_xml(xml: &str) -> Result<Vec<String>, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut definitions = Vec::new();
    let mut current = String::new();
    let mut inside_definition = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) if event.name().as_ref() == b"def" => {
                inside_definition = true;
                current.clear();
            }
            Ok(Event::End(event)) if event.name().as_ref() == b"def" => {
                inside_definition = false;
                let definition = current.split_whitespace().collect::<Vec<_>>().join(" ");
                if !definition.is_empty() {
                    definitions.push(definition.chars().take(600).collect());
                }
            }
            Ok(Event::Text(text)) if inside_definition => {
                let value = text
                    .decode()
                    .map_err(|error| format!("invalid dictionary XML: {error}"))?;
                current.push_str(&value);
                current.push(' ');
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(format!("invalid dictionary XML: {error}")),
            _ => {}
        }
    }
    Ok(definitions)
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_explicit_portuguese_dictionary_queries() {
        assert_eq!(
            query_word("definir resiliência").as_deref(),
            Some("resiliência")
        );
        assert_eq!(query_word("significado de casa").as_deref(), Some("casa"));
        assert!(query_word("abrir casa").is_none());
    }

    #[test]
    fn extracts_bounded_definitions_from_tei_xml() {
        let xml = "<entry><sense><def>Capacidade de se adaptar.</def><def>Resistência.</def></sense></entry>";
        assert_eq!(
            definitions_from_xml(xml).unwrap(),
            ["Capacidade de se adaptar.", "Resistência."]
        );
    }
}
