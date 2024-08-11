//! UU 在线工具 https://uutool.cn/ipv6/

use std::sync::LazyLock;

use anyhow::anyhow;
use reqwest::header::{self, HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    util::{padding_ipv6, wait_blink},
    IPRegion,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UUToolResponse {
    pub code: i64,
    pub data: Option<Data>,
    pub request_id: String,
    pub error: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub ip: String,
    pub ip_int: Value,
    pub location: String,
    pub continent: String,
    pub country: String,
    pub country_code: String,
    pub province: String,
    pub city: String,
    pub district: String,
    pub street: String,
    pub isp: String,
    pub latitude: String,
    pub longitude: String,
    pub area_code: String,
    pub zip_code: String,
    pub time_zone: String,
    pub street_history: Vec<Value>,
    pub risk: Risk,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Risk {
    pub risk_score: i64,
    pub risk_level: String,
    pub is_proxy: String,
    pub proxy_type: String,
    pub risk_tag: String,
}

static HEADERS: LazyLock<HeaderMap> = LazyLock::new(|| {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "accept",
        "application/json, text/javascript, */*; q=0.01"
            .parse()
            .unwrap(),
    );
    headers.insert("accept-language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert(
        "content-type",
        "application/x-www-form-urlencoded; charset=UTF-8"
            .parse()
            .unwrap(),
    );
    headers.insert("dnt", "1".parse().unwrap());
    headers.insert("origin", "https://uutool.cn".parse().unwrap());
    headers.insert("priority", "u=1, i".parse().unwrap());
    headers.insert("referer", "https://uutool.cn/".parse().unwrap());
    headers.insert(
        "sec-ch-ua",
        "\"Not)A;Brand\";v=\"99\", \"Google Chrome\";v=\"127\", \"Chromium\";v=\"127\""
            .parse()
            .unwrap(),
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
    headers.insert("sec-fetch-dest", "empty".parse().unwrap());
    headers.insert("sec-fetch-mode", "cors".parse().unwrap());
    headers.insert("sec-fetch-site", "same-site".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".parse().unwrap());
    headers
});

async fn query_ipv6(ip: &str) -> Result<UUToolResponse, anyhow::Error> {
    // 填充ipv6地址
    let mut ip = ip.to_string();
    ip = padding_ipv6(&ip);
    let response = reqwest::Client::new()
        .post("https://api.ip77.net/ip2/v4/")
        .headers((*HEADERS).clone())
        .body(format!("ip={ip}"))
        .send()
        .await
        .unwrap()
        .json::<UUToolResponse>()
        .await
        .unwrap();
    Ok(response)
}

pub async fn query_uutool(ip: &str) -> Result<IPRegion, anyhow::Error> {
    let wait = wait_blink("查询中，请稍候🔎...", 3);
    let res = query_ipv6(ip).await?;
    wait.sender.send(true).unwrap();
    wait.handle.await?;
    if res.code != 0 {
        let mut msg = "查询出错".to_string();
        if let Some(e) = res.error {
            msg = e;
        }
        return Err(anyhow!("{}", msg));
    }
    let data = res.data.unwrap();
    Ok(IPRegion::new(ip.to_string(), data.location, Some(data.isp)))
}
