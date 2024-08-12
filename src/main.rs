use std::{
    fs as std_fs,
    io::{self, Write},
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
    time::{self, Duration},
};

use clap::Parser;
use colored::Colorize;
use iprr::{
    ip2region::{XDB_FILEPATH, XDB_URL},
    qqwry::{QQWRY_FILEPATH, QQWRY_URL},
    util::{clear_current_line, clear_prev_line, download_file, replace_home, wait_blink},
    Search, Searcher,
};
use tokio::{fs, io::AsyncWriteExt, sync::mpsc, time::sleep};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// 使用 https://uutool.cn 提供的在线查询
    #[clap(short, long, conflicts_with_all = vec!["ip2region", "ip138", "all"])]
    uutool: bool,

    /// 使用 https://ip138.com 提供的在线查询
    #[clap(long, conflicts_with_all = vec!["uutool", "ip2region", "all"])]
    ip138: bool,

    /// 使用ip2region提供的离线查询
    #[clap(short, long, conflicts_with_all = ["uutool", "ip138", "all"])]
    ip2region: bool,

    /// ip2region离线数据库文件所在地址, 默认 ~/.cache/ipr/ip2region.xdb
    #[arg(long)]
    ip2region_db_path: Option<String>,

    /// 更新ip2region离线数据库
    #[clap(long, conflicts_with_all = vec!["uutool", "ip138", "all"])]
    ip2region_update: bool,

    /// ip2region离线数据库更新链接, 默认 https://cdn.jsdelivr.net/gh/lionsoul2014/ip2region/data/ip2region.xdb
    #[arg(long)]
    ip2region_update_url: Option<String>,

    /// 纯真离线数据库文件所在地址, 默认 ~/.cache/ipr/qqwry.dat
    #[arg(long)]
    qqwry_db_path: Option<String>,

    /// 更新纯真离线数据库
    #[clap(long, conflicts_with_all = vec!["uutool", "ip138", "ip2region", "all"])]
    qqwry_update: bool,

    /// 纯真离线数据库更新链接, 默认 https://raw.githubusercontent.com/FW27623/qqwry/main/qqwry.dat
    #[arg(long)]
    qqwry_update_url: Option<String>,

    /// 查询所有渠道
    #[clap(short, long, conflicts_with_all = vec!["uutool", "ip2region", "ip138"])]
    all: bool,

    /// IP地址, 支持IPv4和IPv6(离线模式不支持)
    ip: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let start = time::Instant::now();
    let cli = Cli::parse();
    // 更新离线数据库
    if cli.ip2region_update || cli.qqwry_update {
        let (download_url, download_dest) = if cli.ip2region_update {
            (
                cli.ip2region_update_url.unwrap_or(XDB_URL.to_string()),
                (*XDB_FILEPATH).clone(),
            )
        } else {
            (
                cli.qqwry_update_url.unwrap_or(QQWRY_URL.to_string()),
                PathBuf::from(replace_home(QQWRY_FILEPATH)),
            )
        };
        let wait = wait_blink("更新中, 请稍候🚀...", 3);
        download_file(&download_url, &download_dest).await?;
        wait.sender.send(true).unwrap();
        wait.handle.await?;
        println!(
            "{} {}",
            "更新成功 ✅".green().bold(),
            format!("{}ms elapsed.", start.elapsed().as_millis()).bright_black()
        );
        return Ok(());
    }
    let ip = match cli.ip {
        Some(ip) => {
            if !ip.contains(":") && ip.split(".").collect::<Vec<&str>>().len() != 4 {
                eprintln!("{}: {}", "IP格式错误".red(), ip);
                exit(1);
            }
            ip
        }
        None => {
            let (tx, mut rx) = mpsc::channel(1);
            tokio::spawn(async move {
                loop {
                    let mut input = String::new();
                    io::stdout().flush().unwrap();
                    match io::stdin().read_line(&mut input) {
                        Ok(size) => {
                            if size == 0 {
                                break;
                            }
                            if let Err(e) = tx
                                .send((
                                    input.trim().to_lowercase(),
                                    is_terminal::is_terminal(io::stdin()),
                                ))
                                .await
                            {
                                eprintln!(
                                    "{} {}",
                                    "Something went wrong 😭".red(),
                                    e.to_string().red()
                                );
                                exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "{} {}",
                                "Something went wrong 😭".red(),
                                e.to_string().red()
                            );
                            exit(1);
                        }
                    }
                }
            });
            let mut searcher = Searcher::new(iprr::SearchProviderEnum::QQWry(Some(QQWRY_FILEPATH)));
            // 等待20ms，从pipe读取数据完成
            sleep(Duration::from_millis(20)).await;
            if let Ok(input) = rx.try_recv() {
                searcher.search_print(&input.0, true, false).await?;
                return Ok(());
            }
            println!(
                "{} {}",
                "🌍欢迎使用IP归属地查询, 请输入IP.".magenta(),
                "输入help查看帮助.".bright_black(),
            );
            // 读取上一个ip
            let last_ip_file = PathBuf::from(replace_home("~/.cache/ipr/last_ip.dat"));
            let mut last_ip = if last_ip_file.exists() {
                fs::read_to_string(last_ip_file.clone())
                    .await
                    .unwrap()
                    .to_string()
            } else {
                String::new()
            };
            let last_ip_arc = Arc::new(Mutex::new(last_ip.clone()));
            {
                let last_ip_arc = last_ip_arc.clone();
                let last_ip_file = last_ip_file.clone();
                ctrlc::set_handler(move || {
                    let last_ip_file = last_ip_file.clone();
                    if !last_ip_file.parent().unwrap().exists() {
                        std_fs::create_dir_all(last_ip_file.parent().unwrap()).unwrap();
                    }
                    let mut file = std_fs::File::create(last_ip_file).unwrap();
                    let last_ip = last_ip_arc.lock().unwrap().to_string();
                    file.write_all(last_ip.as_bytes()).unwrap();
                    println!("\nBye !");
                    exit(0);
                })
                .unwrap();
            }
            // 上次输入
            let mut last_input = String::new();
            // 是否来自pipe
            let mut is_pipe = false;
            loop {
                print!(">>> ");
                io::stdout().flush().unwrap();
                let input = rx.recv().await;
                if input.is_none() {
                    if is_pipe {
                        clear_current_line();
                        if !last_input.is_empty() {
                            clear_prev_line();
                            clear_prev_line();
                            searcher.search_print(&last_input, true, false).await?;
                        } else {
                            eprintln!("[ERR] {}", "input from pipe is empty".red());
                        }
                        exit(0);
                    } else {
                        eprintln!("[ERR] {}", "channel is close".red());
                        exit(1);
                    }
                }
                let input = input.unwrap();
                is_pipe = !input.1;
                let input = input.0;
                if input.is_empty() {
                    continue;
                } else if input == "q" || input == "exit" || input == "quit" {
                    if !last_ip.is_empty() {
                        if !last_ip_file.parent().unwrap().exists() {
                            fs::create_dir_all(last_ip_file.parent().unwrap()).await?;
                        }
                        let mut file = fs::File::create(last_ip_file).await?;
                        file.write_all(last_ip.as_bytes()).await?;
                    }
                    break;
                } else if input == "help" || input == "h" {
                    println!("1.输入IP地址, 按回车提交查询.纯真数据库(qqwry)及uutool支持查询IPv6, 其余只支持IPv4.");
                    println!("2.默认查询纯真数据库(qqwry), 输入`select channel`切换渠道, 变量channel支持`ip138`, `ip2region`, `uutool`及`qqwry`.");
                    println!("3.输入`info`或`i`查看当前查询渠道.");
                    println!("4.输入`!!`重复上一次查询.");
                    println!("5.输入`quit`,`q`或`exit`退出查询.");
                    println!("6.输入`help`或`h`查看帮助.");
                    continue;
                } else if input.starts_with("select") {
                    match input.split(" ").last() {
                        Some(channel) => {
                            let channel = channel.trim();
                            if channel == "ip138" {
                                searcher = Searcher::new(iprr::SearchProviderEnum::IP138);
                            } else if channel == "ip2region" {
                                searcher = Searcher::new(iprr::SearchProviderEnum::IP2Region(None));
                            } else if channel == "uutool" {
                                searcher = Searcher::new(iprr::SearchProviderEnum::UUTool);
                            } else if channel == "qqwry" {
                                searcher = Searcher::new(iprr::SearchProviderEnum::QQWry(None));
                            } else {
                                eprintln!("{}: {channel}", "渠道参数错误".red());
                                continue;
                            }
                            println!(
                                "已切换到 {}",
                                searcher.search_provider.get_source().magenta()
                            );
                        }
                        None => {
                            eprintln!("{}", "渠道参数错误！".red());
                        }
                    }
                    continue;
                } else if input == "info" || input == "i" {
                    println!(
                        "当前查询渠道 {}",
                        searcher.search_provider.get_source().magenta(),
                    );
                    continue;
                } else if input == "!!" {
                    if last_ip.is_empty() {
                        eprintln!("{}", "未找到上次查询的IP".red());
                        continue;
                    }
                    println!(">>> {}", last_ip.bright_black());
                    searcher.search_print(&last_ip, false, false).await?;
                    continue;
                }
                last_ip = input.clone();
                last_input = last_ip.clone();
                let last_ip_arc = Arc::clone(&last_ip_arc);
                {
                    let mut s = last_ip_arc.lock().unwrap();
                    *s = last_ip.clone();
                }
                searcher.search_print(&input, false, false).await?;
            }
            println!("Bye!");
            exit(0);
        }
    };
    let mut query_all = false;
    let ip2region_db_path = match cli.ip2region_db_path {
        Some(e) => e,
        None => XDB_FILEPATH.to_str().unwrap().to_string(),
    };
    let qqwry_db_path = match cli.qqwry_db_path {
        Some(e) => e,
        None => QQWRY_FILEPATH.to_string(),
    };
    let mut searcher = if cli.uutool {
        Searcher::new(iprr::SearchProviderEnum::UUTool)
    } else if cli.ip2region {
        Searcher::new(iprr::SearchProviderEnum::IP2Region(Some(
            &ip2region_db_path,
        )))
    } else if cli.ip138 {
        Searcher::new(iprr::SearchProviderEnum::IP138)
    } else if cli.all {
        query_all = true;
        Searcher::new(iprr::SearchProviderEnum::ALL)
    } else {
        Searcher::new(iprr::SearchProviderEnum::QQWry(Some(&qqwry_db_path)))
    };
    searcher.search_print(&ip, true, query_all).await?;
    println!(
        "{} {}",
        searcher.search_provider.get_source().bright_black(),
        format!("{}ms elapsed.", start.elapsed().as_millis()).bright_black(),
    );
    Ok(())
}
