mod args;
mod docker;
mod interfaces;

use std::{thread::sleep, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    args::{limit_to_byte, parse_args},
    docker::{exec, is_running_in_docker},
    interfaces::get_interface_rtx,
};

const DIR_LIMIT_HISTORY: &'static str = "/limit";
const PATH_LIMIT_HISTORY: &'static str = "/limit/limit.json";

#[derive(Serialize, Deserialize, Default, Debug)]
struct History {
    /// 计流周期, 如: "2025-06-01"
    cycle: String,
    /// 本次开机之前流量
    before: i128,
    /// 本次开机时间
    uptime: String,
    /// 本次开机实时流量
    this_time: i128,
    /// 本周期总流量
    total: String,
}

fn main() -> std::io::Result<()> {
    let Some(args) = parse_args() else {
        return Ok(());
    };

    let interface = &args.interface;
    let limit = &args.limit;
    let reset_day = args.reset_day;
    macro_log::i!(
        "interface: {}, limit: {}, reset_day: {}",
        interface,
        limit,
        reset_day
    );
    macro_log::i!("docker: {}", is_running_in_docker());
    let limit = limit_to_byte(limit) as i128;

    let output = exec("uptime", &mut vec!["-s"])?;
    let uptime = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let json = std::fs::read_to_string(PATH_LIMIT_HISTORY).unwrap_or_default();
    let mut history: History = serde_json::from_str(&json).unwrap_or_default();
    macro_log::i!("{:?}", history);
    // 开机时间不同, 鉴定为机器重启, 重新累计流量
    if uptime != history.uptime {
        history.uptime = uptime;
        history.before += history.this_time;
        history.this_time = 0; // 待查询
    }
    // 处理周期
    if history.cycle.is_empty() {
        let output = exec("date", &mut vec!["-I"])?;
        let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
        history.cycle = date;
    }
    macro_log::i!("{:?}", history);

    if !std::fs::exists(DIR_LIMIT_HISTORY)? {
        std::fs::create_dir(DIR_LIMIT_HISTORY)?;
    }

    let regex = regex::Regex::new(&format!(
        r#"-{}{}$"#,
        if reset_day < 10 { "0" } else { "" },
        reset_day
    ))
    .unwrap();

    loop {
        let Ok(rtx) = get_interface_rtx(interface) else {
            macro_log::e!("Failed to get_interface_rtx: {}", interface);
            return Ok(());
        };
        // 实时累计流量 = 历史流量 + 本次开机以来流量
        let total = history.before + rtx;
        macro_log::i!(
            "rtx: {:.2} GB, total: {:.2} GB",
            rtx as f64 / 1024.0 / 1024.0 / 1024.0,
            total as f64 / 1024.0 / 1024.0 / 1024.0
        );
        // 距离上次时写入记录文件时的流量
        let diff = rtx - history.this_time;
        // 流量超过 1MB 或即将关机, 写入记录文件
        if diff > (1.0 * 1024.0 * 1024.0) as i128 || history.before + rtx > limit {
            history.this_time = rtx;
            history.total = format!("{:.2} GB", total as f64 / 1024.0 / 1024.0 / 1024.0);
            let value = serde_json::to_string_pretty(&history)?;
            std::fs::write(PATH_LIMIT_HISTORY, value)?;
        }
        if total > limit {
            macro_log::w!("流量超出限制，关机中...");
            let output = exec("poweroff", &mut vec![])?;
            output
                .status
                .success()
                .then(|| {
                    macro_log::i!(
                        "关机命令执行成功:\n{}",
                        String::from_utf8_lossy(&output.stdout)
                    );
                })
                .unwrap_or_else(|| {
                    macro_log::e!(
                        "关机命令执行失败:\n{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                });
        }
        sleep(Duration::from_secs(10));

        let output = exec("date", &mut vec!["-I"])?;
        let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if regex.is_match(&date) && date != history.cycle {
            macro_log::i!("周期日到，重置流量");
            history.cycle = date;
            history.before = -rtx;
            history.this_time = rtx;
            history.total = format!("{:.2} GB", 0);
            let value = serde_json::to_string_pretty(&history)?;
            std::fs::write(PATH_LIMIT_HISTORY, value)?;
        }
    }
}
