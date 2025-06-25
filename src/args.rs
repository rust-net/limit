use clap::Parser;
use regex::Regex;

use crate::interfaces::help_interfaces;

/// 流量超出限制自动关机程序
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 网络接口名
    #[arg(short, long)]
    pub interface: String,

    /// 流量限制大小
    #[arg(short, long, default_value = "50G", value_parser = validate_limit)]
    pub limit: String,
}

fn validate_limit(s: &str) -> Result<String, String> {
    let regex = Regex::new(r"^\d+(\.?\d+)?[KMG]B?$").unwrap();
    if regex.is_match(s) {
        Ok(s.to_string())
    } else {
        Err(format!("无效的流量大小格式: {}", s))
    }
}

pub fn limit_to_byte(limit: &str) -> f64 {
    let regex = Regex::new(r"^(\d+(\.?\d+)?)([KMG])B?$").unwrap();
    let caps = regex.captures(limit).unwrap();
    match caps.get(3).unwrap().as_str() {
        "K" => 1024.0 * caps.get(1).unwrap().as_str().parse::<f64>().unwrap(),
        "M" => 1024.0 * 1024.0 * caps.get(1).unwrap().as_str().parse::<f64>().unwrap(),
        "G" => 1024.0 * 1024.0 * 1024.0 * caps.get(1).unwrap().as_str().parse::<f64>().unwrap(),
        _ => 0.0,
    }
}

pub fn parse_args() -> Option<Args> {
    let args = match Args::try_parse() {
        Ok(r) => r,
        Err(e) => {
            if let Some(miss_args) = e.get(clap::error::ContextKind::InvalidArg) {
                if let clap::error::ContextValue::Strings(args) = miss_args {
                    if args.iter().any(|it| it.eq("--interface <INTERFACE>")) {
                        help_interfaces();
                        return None;
                    }
                }
            }
            let _ = e.print();
            return None;
        }
    };
    Some(args)
}

#[test]
fn test_limit_to_byte() {
    assert_eq!(limit_to_byte("1.0KB"), 1024.0);
    assert_eq!(limit_to_byte("1.1KB"), 1126.4);
    assert_eq!(limit_to_byte("1.1111MB"), 1024.0 * 1024.0 * 1.1111);
    assert_eq!(limit_to_byte("100GB"), 100.0 * 1024.0 * 1024.0 * 1024.0);
}
