#![allow(deprecated)]

use colored::Colorize;

use tokio::{
    fs::{self},
    io as t_io,
};

use std::{env, io, path::Path};
use std::{io::Write, time::Duration};
use tokio::{
    sync::oneshot::{self, Sender},
    task::JoinHandle,
    time::sleep,
};

pub fn clear_current_line() {
    // 使用 ANSI 转义序列清除行并将光标移到行首
    print!("\r\x1B[2K");
    io::stdout().flush().unwrap();
}

pub fn clear_prev_line() {
    print!("\x1b[1A");
    print!("\x1b[2K");
    io::stdout().flush().unwrap();
}

#[derive(Debug)]
pub struct WaitBlinker {
    pub sender: Sender<bool>,
    pub handle: JoinHandle<()>,
}

pub fn wait_blink(msg: &str, blink_char_num: usize) -> WaitBlinker {
    let msg = msg.to_string();
    let (tx, mut rx) = oneshot::channel::<bool>();
    let handle = tokio::spawn(async move {
        loop {
            print!("{}", format!("\r{}", msg).green());
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(120)).await;
            print!(
                "{}",
                format!(
                    "\r{}{}",
                    msg.chars()
                        .take(msg.chars().count() - blink_char_num)
                        .collect::<String>(),
                    " ".repeat(blink_char_num),
                )
                .green()
            );
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(50)).await;
            if rx.try_recv().is_ok() {
                clear_current_line();
                break;
            }
        }
    });
    WaitBlinker { sender: tx, handle }
}

// 下载文件
pub async fn download_file(download_url: &str, dest: &Path) -> Result<(), anyhow::Error> {
    let response = reqwest::get(download_url).await?;
    let dest_dir = dest.parent().unwrap();
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir).await?;
    }
    t_io::copy(
        &mut response.bytes().await?.as_ref(),
        &mut fs::File::create(dest).await?,
    )
    .await?;
    Ok(())
}

pub fn replace_home(p: &str) -> String {
    if p.starts_with('~') {
        let home = env::home_dir().unwrap();
        return p.replace("~", home.to_str().unwrap());
    }
    p.to_string()
}

pub fn padding_ipv6(ip: &str) -> String {
    if ip.contains("::") {
        let repeat = 8 - ip.split(':').filter(|x| !(*x).is_empty()).count();
        let mut ip = ip
            .replace("::", ":0:".repeat(repeat).as_str())
            .replace("::", ":");
        if ip.ends_with(":") {
            ip = ip.strip_suffix(":").unwrap().into();
        }
        return ip;
    }
    ip.to_string()
}
