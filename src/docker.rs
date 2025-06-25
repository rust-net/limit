use std::process::Command;

/// 检测当前程序是否在 Docker 容器中运行
pub fn is_running_in_docker() -> bool {
    use std::fs;

    // 检查 /.dockerenv 文件是否存在
    if fs::metadata("/.dockerenv").is_ok() {
        return true;
    }

    // 检查 /proc/1/cgroup 文件内容
    if let Ok(contents) = fs::read_to_string("/proc/1/cgroup") {
        if contents.contains("docker") || contents.contains("kubepods") {
            return true;
        }
    }

    false
}

pub fn exec<'a>(exe: &'a str, param: &mut Vec<&'a str>) -> std::io::Result<std::process::Output> {
    if is_running_in_docker() {
        let mut params = vec!["/host", exe];
        params.append(param);
        Command::new("chroot").args(params).output()
    } else {
        let mut params = vec![];
        params.append(param);
        Command::new(exe).args(params).output()
    }
}