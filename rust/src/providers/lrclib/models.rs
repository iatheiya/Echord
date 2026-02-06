// Файл: models.rs
// Определения структур данных, моделей API и логики парсера
// [АУДИТ 5.3] LrcLibError УДАЛЕН и перенесен в core/error.rs

use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
// LrcLibError и thiserror::Error больше не нужны здесь

// ## Ошибки FFI ##
// (Перемещено в core/error.rs)

// ## Модели Данных ##

/// Модель ответа от API lrclib.net (Rust-internal)
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: i64,
    pub track_name: String,
    pub artist_name: String,
    pub duration: f64,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

impl Track {
    /// Воспроизводит ленивое свойство `lrc` из Kotlin
    /// Парсит `syncedLyrics` по требованию
    pub fn lrc(&self) -> Option<LrcFile> {
        self.synced_lyrics
            .as_ref()
            .and_then(|text| parser::parse(text))
            .map(parser::to_lrc_file)
    }
}

/// Финальная структура текста песни, возвращаемая через FFI
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Lyrics {
    pub text: String,
    pub synced: bool,
}

impl Lyrics {
    /// Воспроизводит метод `asLrc()` из Kotlin
    pub fn as_lrc(&self) -> Option<LrcFile> {
        if !self.synced {
            return None;
        }
        parser::parse(&self.text).map(parser::to_lrc_file)
    }
}

/// Представление распарсенной строки LRC
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LrcLine {
    Invalid,
    Metadata {
        key: String,
        value: String,
    },
    Lyric {
        /// Timestamp в миллисекундах
        timestamp: u64,
        line: String,
    },
}

/// Представление целого LRC-файла
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LrcFile {
    pub metadata: HashMap<String, String>,
    /// Карта <Timestamp (ms), Текст строки>
    pub lines: HashMap<u64, String>,
    pub errors: usize,
}

// Хелперы для доступа к метаданным, как в Kotlin
impl LrcFile {
    fn get_meta(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn title(&self) -> Option<&String> {
        self.get_meta("ti")
    }
    pub fn artist(&self) -> Option<&String> {
        self.get_meta("ar")
    }
    pub fn album(&self) -> Option<&String> {
        self.get_meta("al")
    }
    pub fn author(&self) -> Option<&String> {
        self.get_meta("au")
    }
    pub fn file_author(&self) -> Option<&String> {
        self.get_meta("by")
    }
    pub fn tool(&self) -> Option<&String> {
        self.get_meta("re").or_else(|| self.get_meta("tool"))
    }
    pub fn version(&self) -> Option<&String> {
        self.get_meta("ve")
    }

    pub fn duration(&self) -> Option<Duration> {
        self.get_meta("length").and_then(|s| {
            let parts: Vec<_> = s.split(':').collect();
            if parts.len() == 2 {
                let minutes = parts[0].parse::<u64>().ok()?;
                let seconds = parts[1].parse::<u64>().ok()?;
                Some(Duration::from_secs(minutes * 60 + seconds))
            } else {
                None
            }
        })
    }

    pub fn offset(&self) -> Option<Duration> {
        self.get_meta("offset")
            .and_then(|s| s.trim_start_matches('+').parse::<i64>().ok())
            .map(|ms| Duration::from_millis(ms as u64))
    }
}

// ## Модуль Парсера (внутренний) ##

mod parser {
    use super::{LrcFile, LrcLine};
    use once_cell::sync::Lazy;
    use regex::Regex;
    use std::collections::HashMap;

    // АУДИТ 4.2 (Performance): Regex компилируется один раз,
    // что является оптимальным (Правило 1.2 - Унификация).
    static LYRIC_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\[(\d{2,}):(\d{2})\.(\d{2,3})](.*)$").unwrap());
    static METADATA_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\[(.+?):(.*?)]$").unwrap());

    /// Порт `LrcParser.parse`
    pub fn parse(raw: &str) -> Option<Vec<LrcLine>> {
        let lines: Vec<LrcLine> = raw
            .lines()
            .filter_map(|line| {
                // Обработка комментариев '#' (как в Kotlin)
                line.split_once('#')
                    .map_or(line, |(before, _)| before)
                    .trim()
                    .take_if(|s| !s.is_empty())
            })
            .map(|line| {
                // Попытка 1: Спарсить как строку текста
                if let Some(caps) = LYRIC_REGEX.captures(line) {
                    if let (Some(min), Some(sec), Some(mil), Some(text)) =
                        (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                    {
                        // АУДИТ 2.3 (Семантика): Логика `padEnd(3, '0')` из Kotlin
                        // "12" -> "120", "1" -> "100", "123" -> "123"
                        let millis_str = format!("{:0<3}", mil.as_str());

                        if let (Ok(min), Ok(sec), Ok(mil)) = (
                            min.as_str().parse::<u64>(),
                            sec.as_str().parse::<u64>(),
                            millis_str.parse::<u64>(),
                        ) {
                            let timestamp = (min * 60 * 1000) + (sec * 1000) + mil;
                            return LrcLine::Lyric {
                                timestamp,
                                line: text.as_str().trim().to_string(),
                            };
                        }
                    }
                }

                // Попытка 2: Спарсить как метаданные
                if let Some(caps) = METADATA_REGEX.captures(line) {
                    if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
                        return LrcLine::Metadata {
                            key: key.as_str().trim().to_string(),
                            value: value.as_str().trim().to_string(),
                        };
                    }
                }

                // Не удалось спарсить
                LrcLine::Invalid
            })
            .collect();

        // Проверка, как в Kotlin
        if lines.is_empty() || lines.iter().all(|l| matches!(l, LrcLine::Invalid)) {
            None
        } else {
            Some(lines)
        }
    }

    /// Порт `List<LrcParser.Line>.toLrcFile()`
    pub fn to_lrc_file(lines: Vec<LrcLine>) -> LrcFile {
        let mut metadata = HashMap::new();
        let mut lyric_lines = HashMap::new();
        let mut errors = 0;

        // АУДИТ 2.3 (Семантика): В Kotlin `lines` начинаются с `0L to ""`.
        // Это поведение воспроизведено.
        lyric_lines.insert(0, "".to_string());

        for line in lines {
            match line {
                LrcLine::Invalid => errors += 1,
                LrcLine::Metadata { key, value } => {
                    metadata.insert(key, value);
                }
                LrcLine::Lyric { timestamp, line } => {
                    lyric_lines.insert(timestamp, line);
                }
            }
        }

        LrcFile {
            metadata,
            lines: lyric_lines,
            errors,
        }
    }

    // ## Тесты (Правило 1.4) ##
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_metadata() {
            let raw = "[ar:Artist]\n[ti:Title]";
            let lines = parse(raw).unwrap();
            assert_eq!(
                lines[0],
                LrcLine::Metadata {
                    key: "ar".to_string(),
                    value: "Artist".to_string()
                }
            );
        }

        #[test]
        fn test_parse_lyrics_and_padding() {
            // Тест на `padEnd(3, '0')`. .12 -> 120ms
            let raw = "[00:01.12]Line 1\n[00:01.123]Line 2";
            let lines = parse(raw).unwrap();
            assert_eq!(
                lines[0],
                LrcLine::Lyric {
                    timestamp: 1120, // 1s + 120ms
                    line: "Line 1".to_string()
                }
            );
            assert_eq!(
                lines[1],
                LrcLine::Lyric {
                    timestamp: 1123, // 1s + 123ms
                    line: "Line 2".to_string()
                }
            );
        }

        #[test]
        fn test_to_lrc_file() {
            let lines = vec![
                LrcLine::Metadata {
                    key: "ti".to_string(),
                    value: "Test".to_string(),
                },
                LrcLine::Lyric {
                    timestamp: 500,
                    line: "Hello".to_string(),
                },
                LrcLine::Invalid,
            ];
            let file = to_lrc_file(lines);
            assert_eq!(file.title(), Some(&"Test".to_string()));
            assert_eq!(file.lines.get(&0), Some(&"".to_string())); // Проверка начальной строки
            assert_eq!(file.errors, 1);
        }
    }
}
