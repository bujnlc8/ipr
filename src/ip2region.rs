//! https://github.com/lionsoul2014/ip2region

use std::{path::PathBuf, sync::LazyLock};

use anyhow::anyhow;
use xdb::{search_by_ip, searcher_init};

use crate::{
    util::{download_file, replace_home, wait_blink},
    IPRegion,
};

// https://raw.githubusercontent.com/lionsoul2014/ip2region/master/data/ip2region.xdb
pub const XDB_URL: &str = "https://cdn.jsdelivr.net/gh/lionsoul2014/ip2region/data/ip2region.xdb";

pub static XDB_FILEPATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let p = replace_home("~/.cache/ipr/ip2region.xdb");
    PathBuf::from(p)
});

pub async fn query_ip2region(ip: &str, xdb_path: Option<&str>) -> Result<IPRegion, anyhow::Error> {
    let wait = wait_blink("查询中，请稍候🔎...", 3);
    let xdb_path = replace_home(xdb_path.unwrap_or(XDB_FILEPATH.to_str().unwrap()));
    if !PathBuf::from(xdb_path.clone()).exists() {
        download_file(XDB_URL, &XDB_FILEPATH).await?;
        searcher_init(Some(XDB_FILEPATH.to_str().unwrap().to_string()));
    } else {
        searcher_init(Some(xdb_path));
    }
    wait.sender.send(true).unwrap();
    wait.handle.await?;
    if ip.contains(":") {
        return Err(anyhow!("暂不支持IPv6"));
    }
    match search_by_ip(ip) {
        Ok(r) => {
            let r = r
                .split('|')
                .filter(|x| *x != "0")
                .collect::<Vec<&str>>()
                .join("");
            Ok(IPRegion::new(ip.to_string(), r, None))
        }
        Err(e) => Err(anyhow!(e.to_string())),
    }
}
