mod ip138;
pub mod ip2region;
pub mod qqwry;
pub mod util;
mod uutool;
mod zxipv6;

use anyhow::anyhow;
use colored::Colorize;
use ip138::query_ip138;
use ip2region::query_ip2region;
use qqwry::query_qqwry;
use uutool::query_uutool;

// 查询服务提供方
#[derive(Debug, Clone)]
pub enum SearchProviderEnum<'a> {
    IP138,
    UUTool,
    IP2Region(Option<&'a str>),
    QQWry(Option<&'a str>),
    ALL,
}

impl<'a> SearchProviderEnum<'a> {
    pub fn get_source(&self) -> String {
        match self {
            Self::IP138 => "IP138.COM".to_string(),
            Self::UUTool => "UUTOOL.CN".to_string(),
            Self::IP2Region(_) => "IP2REGION".to_string(),
            Self::QQWry(_) => "QQWRY".to_string(),
            Self::ALL => "ALL".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Searcher<'a> {
    pub search_provider: SearchProviderEnum<'a>,
}

pub trait Search {
    fn search_print(
        &mut self,
        ip: &str,
        echo_ip: bool,
        query_all: bool,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    fn search(
        &mut self,
        ip: &str,
    ) -> impl std::future::Future<Output = Result<IPRegion, anyhow::Error>> + Send;
}

impl<'a> Searcher<'a> {
    pub fn new(search_provider: SearchProviderEnum<'a>) -> Self {
        Self { search_provider }
    }
}

impl Search for Searcher<'_> {
    async fn search_print(
        &mut self,
        ip: &str,
        echo_ip: bool,
        query_all: bool,
    ) -> Result<(), anyhow::Error> {
        if !query_all {
            match self.search(ip).await {
                Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                Ok(e) => e.display(echo_ip),
            }
            return Ok(());
        }
        for search_provider in [
            SearchProviderEnum::QQWry(None),
            SearchProviderEnum::IP2Region(None),
            SearchProviderEnum::IP138,
            SearchProviderEnum::UUTool,
        ] {
            match Searcher::new(search_provider.clone()).search(ip).await {
                Ok(e) => {
                    e.display(echo_ip);
                }
                Err(e) => {
                    eprintln!("[ERR] {}.", e.to_string().red());
                }
            }
            println!("{}", search_provider.get_source().bright_black());
        }
        Ok(())
    }

    async fn search(&mut self, ip: &str) -> Result<IPRegion, anyhow::Error> {
        match self.search_provider.clone() {
            SearchProviderEnum::UUTool => query_uutool(ip).await,
            SearchProviderEnum::IP138 => query_ip138(ip).await,
            SearchProviderEnum::QQWry(data_path) => query_qqwry(ip, data_path).await,
            SearchProviderEnum::IP2Region(xdb_path) => query_ip2region(ip, xdb_path).await,
            SearchProviderEnum::ALL => Err(anyhow!("Enum ALL is not implemented")),
        }
    }
}

#[derive(Debug)]
pub struct IPRegion {
    pub ip: String,
    pub region: String,
    pub isp: Option<String>,
}

impl IPRegion {
    pub fn new(ip: String, region: String, isp: Option<String>) -> Self {
        Self { ip, region, isp }
    }

    pub fn display(&self, echo_ip: bool) {
        if echo_ip {
            self._display();
        } else {
            self._display_no_ip();
        }
    }

    fn _display(&self) {
        if let Some(isp) = self.isp.clone() {
            println!(
                "{} [{} {}]",
                self.ip.yellow(),
                self.region.green().bold(),
                isp.green().bold()
            );
        } else {
            println!("{} [{}]", self.ip.yellow(), self.region.green().bold());
        }
    }

    fn _display_no_ip(&self) {
        if let Some(isp) = self.isp.clone() {
            println!("{} {}", self.region.green().bold(), isp.green().bold());
        } else {
            println!("{}", self.region.green().bold());
        }
    }
}
