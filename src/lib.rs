mod ip138;
pub mod ip2region;
pub mod qqwry;
pub mod util;
mod uutool;
mod zxipv6;

use colored::Colorize;
use ip138::query_ip138;
use ip2region::query_ip2region;
use qqwry::query_qqwry;
use uutool::query_uutool;

// 查询服务提供方
#[derive(Debug, Clone)]
pub enum SearchProviderEnum {
    IP138,
    UUTool,
    IP2Region(String),
    QQWry(String),
    ALL,
}

impl SearchProviderEnum {
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
pub struct Searcher {
    pub search_provider: SearchProviderEnum,
}

pub trait Search {
    fn search(
        &mut self,
        ip: &str,
        echo_ip: bool,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}

impl Searcher {
    pub fn new(search_provider: SearchProviderEnum) -> Self {
        Self { search_provider }
    }
}

impl Search for Searcher {
    async fn search(&mut self, ip: &str, echo_ip: bool) -> Result<(), anyhow::Error> {
        match self.search_provider.clone() {
            SearchProviderEnum::UUTool => match query_uutool(ip).await {
                Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                Ok(e) => e.display(echo_ip),
            },
            SearchProviderEnum::IP138 => match query_ip138(ip).await {
                Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                Ok(e) => e.display(echo_ip),
            },
            SearchProviderEnum::IP2Region(xdb_path) => {
                match query_ip2region(ip, Some(&xdb_path)).await {
                    Ok(e) => e.display(echo_ip),
                    Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                }
            }
            SearchProviderEnum::QQWry(data_path) => match query_qqwry(ip, Some(&data_path)).await {
                Ok(e) => e.display(echo_ip),
                Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
            },
            SearchProviderEnum::ALL => {
                match query_ip138(ip).await {
                    Ok(e) => e.display(echo_ip),
                    Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                }
                println!("{}", SearchProviderEnum::IP138.get_source().bright_black());
                match query_uutool(ip).await {
                    Ok(e) => e.display(echo_ip),
                    Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                }
                println!("{}", SearchProviderEnum::UUTool.get_source().bright_black());
                match query_ip2region(ip, None).await {
                    Ok(e) => e.display(echo_ip),
                    Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                }
                println!(
                    "{}",
                    SearchProviderEnum::IP2Region("".to_string())
                        .get_source()
                        .bright_black()
                );
                match query_qqwry(ip, None).await {
                    Ok(e) => e.display(echo_ip),
                    Err(e) => eprintln!("[ERR] {}.", e.to_string().red()),
                }
                println!(
                    "{}",
                    SearchProviderEnum::QQWry("".to_string())
                        .get_source()
                        .bright_black()
                );
            }
        }
        Ok(())
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
