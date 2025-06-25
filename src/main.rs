mod args;
mod docker;
mod interfaces;

use std::{process::Command, thread::sleep, time::Duration};

use crate::{
    args::{limit_to_byte, parse_args},
    docker::is_running_in_docker,
    interfaces::get_interface_rtx,
};

fn main() -> std::io::Result<()> {
    let Some(args) = parse_args() else {
        return Ok(());
    };

    let interface = &args.interface;
    let limit = &args.limit;
    macro_log::i!("interface: {}, limit: {}", interface, limit);
    macro_log::i!("docker: {}", is_running_in_docker());
    let limit = limit_to_byte(limit) as i128;

    loop {
        let Ok(rtx) = get_interface_rtx(interface) else {
            macro_log::e!("Failed to get_interface_rtx: {}", interface);
            return Ok(());
        };
        macro_log::i!("rtx: {:.2} GB", rtx as f64 / 1024.0 / 1024.0 / 1024.0);
        if rtx > limit {
            macro_log::w!("流量超出限制，关机中...");
            let output = if is_running_in_docker() {
                Command::new("chroot").args(["/host", "poweroff"]).output()?
            } else {
                Command::new("poweroff").output()?
            };
            output.status.success().then(|| {
                macro_log::i!("关机命令执行成功:\n{}", String::from_utf8_lossy(&output.stdout));
            }).unwrap_or_else(|| {
                macro_log::e!("关机命令执行失败:\n{}", String::from_utf8_lossy(&output.stderr));
            });
        }
        sleep(Duration::from_secs(10));
    }
}
