//! ip138

use crate::{util::wait_blink, IPRegion};

use anyhow::anyhow;

use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::{collections::HashMap, env, path::PathBuf, sync::LazyLock};
use tokio::fs;

const DEFAULT_CONFIG: &str = r#"
connection->keep-alive
cache-control->max-age=0
sec-ch-ua->"Not A;Brand";v="99", "Chromium";v="96", "Google Chrome";v="96"
sec-ch-ua-mobile->?0
sec-ch-ua-platform->"macOS"
dnt->1
upgrade-insecure-requests->1
user-agent->Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36
accept->text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9
sec-fetch-site->same-origin
sec-fetch-mode->navigate
sec-fetch-user->?1
sec-fetch-dest->document
referer->https://www.ip138.com/iplookup.asp
accept-language->zh,en-US;q=0.9,en;q=0.8,zh-CN;q=0.7
cookie->Hm_lvt_f4f76646cd877e538aa1fbbdf351c548=1639995740; PHPSESSID=hgho5l0972u3dbvq7ivj8ug6ab; ASPSESSIONIDAADBTABA=FDPLEIODAEDOIMPMAJILBJBB; Hm_lpvt_f4f76646cd877e538aa1fbbdf351c548=1639996908
host->www.ip138.com
"#;

static R: &str = "var ip_result = .*;";

static HEADERS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut h = HashMap::new();
    read_into_config(&CONFIG_STR, &mut h);
    h
});

static CONFIG_STR: LazyLock<String> = LazyLock::new(|| {
    let mut s = futures::executor::block_on(read_config());
    if s.is_empty() {
        s = DEFAULT_CONFIG.to_string();
    }
    s
});

fn read_into_config(s: &'static str, m: &mut HashMap<&'static str, &'static str>) {
    let ss: Vec<_> = s.split('\n').filter(|e| !e.is_empty()).collect::<Vec<_>>();
    for k in &ss {
        if k.starts_with('#') {
            continue;
        }
        let d = k.split("->").collect::<Vec<_>>();
        if d.len() != 2 {
            return;
        }
        m.insert(d[0], d[1]);
    }
}

async fn read_config() -> String {
    let config_path = env::var("IPR_CONFIG_PATH").unwrap_or("~/.ipr_config".to_string());
    match fs::read_to_string(PathBuf::from(config_path)).await {
        Ok(e) => e,
        Err(_) => "".to_string(),
    }
}

async fn get_html(
    url: &str,
    headers: &'static HashMap<&str, &str>,
) -> Result<String, anyhow::Error> {
    let client = reqwest::Client::new();
    let mut header = HeaderMap::new();
    for (k, v) in headers {
        header.insert(*k, HeaderValue::from_str(v).unwrap());
    }
    let resp = client.get(url).headers(header).send().await.unwrap();
    if resp.status() != 200 {
        return Ok("".to_string());
    }
    Ok(resp.text_with_charset("gb2312").await?)
}

fn process_field(s: &str) -> String {
    s.replace(['"', ' '], "")
}

pub async fn query_ip138(ip: &str) -> Result<IPRegion, anyhow::Error> {
    let wait = wait_blink("查询中，请稍候🔎...", 3);
    let html_s = get_html(
        format!("https://www.ip138.com/iplookup.asp?ip={}&action=2", ip).as_str(),
        &HEADERS,
    )
    .await?;
    let re = Regex::new(R).unwrap();
    let res = match re.captures(&html_s) {
        Some(e) => e.get(0).map_or("", |m| m.as_str()),
        None => "",
    };
    wait.sender.send(true).unwrap();
    wait.handle.await?;
    if res.is_empty() {
        return Err(anyhow!("未查询到结果！"));
    }

    let res = &res.to_string()[16..];
    let res = res.replace(';', "");
    let v: Value = serde_json::from_str(&res).unwrap();
    Ok(IPRegion::new(
        ip.to_string(),
        process_field(v["ASN归属地"].as_str().unwrap_or("")).to_string(),
        None,
    ))
}
