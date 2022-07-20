//! parser for the returned result from YD

use crate::formatters::Formatter;
use serde_json::{self, Error as SerdeError};

/// Basic result structure
#[derive(Serialize, Deserialize, Debug)]
pub struct YdTranslateResult {
    #[serde(rename = "tgt")]
    target: String,
    #[serde(rename = "src")]
    source: String,
}

/// Web result structure
#[derive(Serialize, Deserialize, Debug)]
pub struct YdSmartResult {
    entries: Vec<String>,
    #[serde(rename = "type")]
    typ: i32,
}

/// Full response structure
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct YdResponse {
    error_code: i32,
    translate_result: Option<Vec<Vec<YdTranslateResult>>>,
    smart_result: Option<YdSmartResult>,
}

impl YdResponse {
    pub fn new_raw(result: String) -> Result<YdResponse, SerdeError> {
        serde_json::from_str(&result)
    }

    /// Explain the result in text format using a formatter
    pub fn explain(&self, fmt: &dyn Formatter) -> String {
        let mut result: Vec<String> = vec![];

        if self.error_code != 0 || self.translate_result.is_none() {
            result.push(fmt.red(" -- No result for this query."));
            return result.join("\n");
        }

        result.push(fmt.cyan("  翻译结果："));

        if let Some(ref translate_result) = self.translate_result {
            for paragraph in translate_result {
                for sentense in paragraph {
                    result.push(format!(
                        "    {}\n    {}",
                        fmt.underline(&sentense.source),
                        sentense.target.clone()
                    ));
                }
            }
        }

        if let Some(ref smart_result) = self.smart_result {
            result.push(fmt.yellow("\n  智能结果："));
            result.push(smart_result.entries.join("    "));
        } else {
            result.push(String::new());
        }

        result.join("\n")
    }
}
