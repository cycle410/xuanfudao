//! 把悬浮岛图标作为 RT_GROUP_ICON #1 编进 exe，供 LoadImageW(hInst, 1, ...) 取用。
//! 优先用 Windows SDK 的 rc.exe；没有 SDK 时退回 mingw 的 windres（GNU 工具链构建）。

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn find_rc() -> Option<PathBuf> {
    let roots = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];
    let mut found: Vec<PathBuf> = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else { continue };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let rc = dir.join("x64").join("rc.exe");
            if rc.exists() {
                found.push(rc);
            }
        }
    }
    found.sort();
    found.pop()
}

/// mingw 的 windres：优先 PATH，其次常见便携安装位置。
fn find_windres() -> Option<PathBuf> {
    if Command::new("windres").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return Some(PathBuf::from("windres"));
    }
    let roots = [r"D:\w64devkit\w64devkit\bin", r"C:\msys64\mingw64\bin"];
    for root in roots {
        let candidate = PathBuf::from(root).join("windres.exe");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/xuanfudao.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // 1) Windows SDK 的 rc.exe（MSVC 工具链）
    if let Some(rc) = find_rc() {
        let out = out_dir.join("xuanfudao.res");
        let status = Command::new(&rc)
            .arg("/nologo")
            .arg("/fo")
            .arg(&out)
            .arg("app.rc")
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            println!("cargo:rustc-link-arg={}", out.display());
            return;
        }
        println!("cargo:warning=rc.exe 编译 app.rc 失败");
    }

    // 2) mingw 的 windres（GNU 工具链，没有 Windows SDK 时）
    if let Some(windres) = find_windres() {
        let obj = out_dir.join("xuanfudao_res.o");
        let status = Command::new(&windres)
            .arg("-O")
            .arg("coff")
            .arg("-i")
            .arg("app.rc")
            .arg("-o")
            .arg(&obj)
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            println!("cargo:rustc-link-arg={}", obj.display());
            return;
        }
        println!("cargo:warning=windres 编译 app.rc 失败");
    }

    println!("cargo:warning=没找到 rc.exe 或 windres，跳过图标和版本资源");
}
