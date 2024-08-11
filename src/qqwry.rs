//! 纯真数据库https://github.com/FW27623/qqwry
use std::path::PathBuf;

use anyhow::anyhow;

use crate::{
    util::{download_file, replace_home, wait_blink},
    zxipv6::query_zxipv6,
    IPRegion,
};

pub const QQWRY_URL: &str = "https://raw.githubusercontent.com/FW27623/qqwry/main/qqwry.dat";

pub const QQWRY_FILEPATH: &str = "~/.cache/ipr/qqwry.dat";

pub async fn query_qqwry(ip: &str, data_path: Option<&str>) -> Result<IPRegion, anyhow::Error> {
    if ip.contains(":") {
        return query_zxipv6(ip).await;
    }
    let wait = wait_blink("查询中，请稍候🔎...", 3);
    let data_path = replace_home(data_path.unwrap_or(QQWRY_FILEPATH));
    if !PathBuf::from(data_path.clone()).exists() {
        download_file(QQWRY_URL, &PathBuf::from(replace_home(QQWRY_FILEPATH))).await?;
    }
    wait.sender.send(true).unwrap();
    wait.handle.await?;
    let client = qqwry::QQWryData::new(PathBuf::from(data_path))?;
    let res = client.query(ip.parse()?);
    match res {
        Some(res) => Ok(IPRegion::new(
            ip.to_string(),
            res.country
                .replace("-", " ")
                .replace("–", "")
                .replace("_", " ")
                .replace("CZ88.NET", "")
                .trim()
                .to_string(),
            Some(
                res.area
                    .replace("-", " ")
                    .replace("–", "")
                    .replace("_", " ")
                    .replace("CZ88.NET", "")
                    .trim()
                    .to_string(),
            ),
        )),
        None => Err(anyhow!("未查询到结果！")),
    }
}
