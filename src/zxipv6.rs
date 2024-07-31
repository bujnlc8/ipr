use std::sync::LazyLock;

use anyhow::anyhow;
use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};

use crate::{util::wait_blink, IPRegion};

static ZX_HEADERS: LazyLock<HeaderMap> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert("accept", "*/*".parse().unwrap());
    headers.insert("accept-language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("dnt", "1".parse().unwrap());
    headers.insert("priority", "u=1, i".parse().unwrap());
    headers.insert("referer", "https://ip.zxinc.org/ipquery/".parse().unwrap());
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
    headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("x-requested-with", "XMLHttpRequest".parse().unwrap());
    headers
});

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    myip: String,
    ip: Ip,
    location: String,
    country: String,
    local: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ip {
    query: String,
    start: String,
    end: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    code: i64,
    data: Option<Data>,
}

async fn _query_zxipv6(ip: &str) -> Result<Response, anyhow::Error> {
    let url = format!("https://ip.zxinc.org/api.php?type=json&ip={}", ip);
    let res = Client::new()
        .get(url)
        .headers((*ZX_HEADERS).clone())
        .send()
        .await?
        .json::<Response>()
        .await?;
    Ok(res)
}

pub async fn query_zxipv6(ip: &str) -> Result<IPRegion, anyhow::Error> {
    let wait = wait_blink("查询中，请稍候🔎...", 3);
    let res = _query_zxipv6(ip).await?;
    wait.sender.send(true).unwrap();
    wait.handle.await?;
    if res.code != 0 {
        return Err(anyhow!("查询出错"));
    }
    let data = res.data.unwrap();
    Ok(IPRegion::new(
        ip.to_string(),
        data.location.replace("\t", " "),
        None,
    ))
}
