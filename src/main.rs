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
    before: i128,
    uptime: String,
    this_time: i128,
}

fn main() -> std::io::Result<()> {
    let Some(args) = parse_args() else {
        return Ok(());
    };

    let interface = &args.interface;
    let limit = &args.limit;
    macro_log::i!("interface: {}, limit: {}", interface, limit);
    macro_log::i!("docker: {}", is_running_in_docker());
    let limit = limit_to_byte(limit) as i128;

    let output = exec("uptime", &mut vec!["-s"])?;
    let uptime = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let json = std::fs::read_to_string(PATH_LIMIT_HISTORY).unwrap_or_default();
    let mut history: History = serde_json::from_str(&json).unwrap_or_default();
    macro_log::i!("{:?}", history);
    // 机器重启, 重新累计流量
    if uptime != history.uptime {
        history.uptime = uptime;
        history.before += history.this_time;
        history.this_time = 0;
    }
    macro_log::i!("{:?}", history);
    std::fs::create_dir(DIR_LIMIT_HISTORY)?;

    loop {
        let Ok(rtx) = get_interface_rtx(interface) else {
            macro_log::e!("Failed to get_interface_rtx: {}", interface);
            return Ok(());
        };
        macro_log::i!("rtx: {:.2} GB", rtx as f64 / 1024.0 / 1024.0 / 1024.0);
        let diff = rtx - history.this_time;
        // 流量超过 1MB 写入记录文件
        if diff > (1.0 * 1024.0 * 1024.0) as i128 {
            history.this_time = rtx;
            let value = serde_json::to_string_pretty(&history)?;
            std::fs::write(PATH_LIMIT_HISTORY, value)?;
        }
        if history.before + rtx > limit {
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
    }
}
