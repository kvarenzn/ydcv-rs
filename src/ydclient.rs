//! ydclient is client wrapper for Client

use serde_json::{self, Error as SerdeError};
use std::collections::HashMap;
use std::env::var;
use std::error::Error;
// use reqwest::header::Connection;
use log::debug;
use super::ydresponse::YdResponse;
use reqwest::blocking::Client;
use reqwest::header::{COOKIE, HOST, ORIGIN, REFERER, USER_AGENT};
use reqwest::Url;

use md5::{Digest, Md5};
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    /// API name
    static ref API: String = var("YDCV_API_NAME")
        .unwrap_or_else(|_| var("YDCV_YOUDAO_APPID")
        .unwrap_or_else(|_| String::from("fanyideskweb")));

    /// API key
    static ref API_KEY: String = var("YDCV_API_KEY")
        .unwrap_or_else(|_| var("YDCV_YOUDAO_APPSEC")
        .unwrap_or_else(|_| String::from("Ygy_4c=r#e#4EX^NUGUc5")));
}

/// Wrapper trait on `reqwest::Client`
pub trait YdClient {
    /// lookup a word on YD and returns a `YdPreponse`
    ///
    /// # Examples
    ///
    /// lookup "hello" and compare the result:
    ///
    /// ```
    /// assert_eq!("YdResponse('hello')",
    ///        format!("{}", Client::new().lookup_word("hello").unwrap()));
    /// ```
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<dyn Error>>;
    fn decode_result(&mut self, result: &str) -> Result<YdResponse, SerdeError>;
}

/// Implement wrapper client trait on `reqwest::Client`
impl YdClient for Client {
    fn decode_result(&mut self, result: &str) -> Result<YdResponse, SerdeError> {
        debug!(
            "Recieved JSON {}",
            serde_json::from_str::<YdResponse>(result)
                .and_then(|v| serde_json::to_string_pretty(&v))
                .unwrap()
        );
        serde_json::from_str(result)
    }

    /// lookup a word on YD and returns a `YdResponse`
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<dyn Error>> {
        use std::io::Read;

        let mut url = Url::parse("https://fanyi.youdao.com/translate_o")?;
        url.query_pairs_mut()
            .extend_pairs([("smartresult", "dict"), ("smartresult", "rule")].iter());

        let api = API.clone();
        let api_key = API_KEY.clone();

        let mut params = HashMap::new();
        params.insert("i", word);
        params.insert("from", "AUTO");
        params.insert("to", "AUTO");
        params.insert("smartresult", "dict");
        params.insert("client", &api);
        params.insert("doctype", "json");
        params.insert("version", "2.1");
        params.insert("keyfrom", "fanyi.web");
        params.insert("action", "FY_BY_DEFAULT");

        let app_version = String::from("5.0 (X11)");
        let mut hasher = Md5::new();
        hasher.update(app_version.as_bytes());
        let bv = format!("{:2x}", hasher.finalize());
        params.insert("bv", &bv);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let timestamp = timestamp.to_string();
        params.insert("lts", &timestamp);

        params.insert("salt", &timestamp);

        let sign = format!("{}{}{}{}", api, word, timestamp, api_key);
        let mut hasher = Md5::new();
        hasher.update(sign.as_bytes());
        let signed = hasher.finalize();
        let signed = format!("{:2x}", signed);
        params.insert("sign", &signed);

        let mut body = String::new();
        self.post(url)
            .header(
                USER_AGENT,
                "Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0",
            )
            .header(HOST, "fanyi.youdao.com")
            .header(ORIGIN, "https://fanyi.youdao.com")
            .header(REFERER, "https://fanyi.youdao.com/")
            .header(COOKIE, "OUTFOX_SEARCH_USER_ID=0@0.0.0.0")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .form(&params)
            .send()?
            .read_to_string(&mut body)?;

        let raw_result = YdResponse::new_raw(body.clone());
        if raw {
            raw_result.map_err(Into::into)
        } else {
            self.decode_result(&body).map_err(Into::into)
        }
    }
}
