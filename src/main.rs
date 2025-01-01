mod mods;
mod patches;
mod utils;

use asky::Text;
use async_std::fs::create_dir;
use godot_pck::structs::PCK;
use std::env::{current_exe, set_current_dir};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use steamlocate::SteamDir;
use sudo::RunningAs;
use sysinfo::ProcessesToUpdate;

static WEBFISHING_APPID: u32 = 3146520;

async fn install_webfishing(location: &SteamDir) {
    let steam_location = location.path();
    let acf_path = steam_location
        .join("steamapps")
        .join(format!("appmanifest_{}.acf", WEBFISHING_APPID));

    println!("Creating Webfishing ACF");
    File::create(acf_path)
        .unwrap()
        .write(include_str!("../res/webfishing.acf").as_bytes())
        .expect("could not write acf");

    println!("Waiting for steam to close");
    let mut system = sysinfo::System::new_all();
    while system.processes_by_name("steam_osx".as_ref()).count() > 0 {
        println!("Steam is still running...");
        async_std::task::sleep(Duration::from_secs(5)).await;
        system.refresh_processes(ProcessesToUpdate::All, true);
    }

    println!("Steam is now closed, please launch it back and wait for webfishing to install");

    while location.find_app(WEBFISHING_APPID).is_err() {
        println!("Steam is not launched...");
        async_std::task::sleep(Duration::from_secs(10)).await;
    }

    println!("Steam launched, downloading webfishing");
    let download_path = steam_location
        .join("steamapps")
        .join("downloading")
        .join(format!("{}", WEBFISHING_APPID));

    while Path::exists(download_path.as_path()) {
        println!("Downloading webfishing...");
        async_std::task::sleep(Duration::from_secs(10)).await;
    }
}

async fn download_godot_steam_template() {
    println!("Downloading GodotSteam template...");
    let res = reqwest::get(
        "https://github.com/GodotSteam/GodotSteam/releases/download/v3.24/macos-g353-s159-gs324.zip",
    )
    .await
    .expect("Could not download godotsteam template");
    let body = res.bytes().await.expect("Could not read body");

    let mut file = File::create("build/godot_steam_template_324.zip")
        .expect("Could not create godotsteam template");
    file.write_all(&body)
        .expect("Could not write godotsteam template");
}

async fn download_gd_decomp() {
    println!("Download Godot Decompiler...");
    let res = reqwest::get("https://github.com/GDRETools/gdsdecomp/releases/download/v0.8.0/GDRE_tools-v0.8.0-macos.zip").await.expect("Could not download decompiler");
    let body = res.bytes().await.expect("Could not read body");

    println!("Unzipping GodotSteam Decompiler...");
    let mut file = File::create("build/decompiler.zip").expect("Could not create decompiler");
    file.write_all(&body).expect("Could not write decompiler");

    Command::new("unzip")
        .arg("decompiler.zip")
        .current_dir("build")
        .output()
        .expect("Could not unzip godotsteam template");
}

fn build_webfishing_macos(webfishing_path: &Path) {
    let template_path = Path::new("build/osx_template.app");
    Command::new("rm")
        .current_dir(template_path)
        .arg("Contents/MacOS/godot_osx_debug.universal")
        .output()
        .expect("Could not remove delete godot_osx_debug.universal");

    Command::new("mv")
        .current_dir(template_path)
        .arg("Contents/MacOS/godot_osx_release.universal")
        .arg("Contents/MacOS/webfishing")
        .output()
        .expect("Could not rename godot_osc_release.universal");

    let mut steamappid = File::create(
        template_path
            .join("Contents")
            .join("MacOS")
            .join("steam_appid.txt"),
    )
    .expect("could not create steam_appid.txt file");
    steamappid
        .write(include_str!("../res/steam_appid.txt").as_bytes())
        .expect("could not write steam_appid.txt");

    Command::new("cp")
        .arg(webfishing_path.join("webfishing.exe"))
        .arg(
            template_path
                .join("Contents")
                .join("Resources")
                .join("webfishing.pck"),
        )
        .output()
        .expect("Could not copy webfishing.exe");

    let mut info_plist = File::create(template_path.join("Contents").join("Info.plist"))
        .expect("Could not open Info.plist");
    info_plist
        .write_all(include_str!("../res/Info.plist").as_bytes())
        .expect("could not write Info.plist");

    Command::new("mv")
        .arg(template_path)
        .arg(Path::new("build/webfishing.app"))
        .output()
        .expect("Could not copy webfishing.app");
}

#[tokio::main]
async fn main() {
    if sudo::check() != RunningAs::Root {
        println!("You need to be root to run this program");
    }
    sudo::escalate_if_needed().expect("Could not escalate");

    set_current_dir(
        current_exe()
            .unwrap()
            .parent()
            .expect("Could not get current dir"),
    )
    .expect("Could not set current dir");
    if !Path::exists("build".as_ref()) {
        println!("Creating build folder");
        create_dir("build")
            .await
            .expect("could not create build folder");
    }

    let location = SteamDir::locate().expect("could not locate steam directory");

    let webfishing = location.find_app(WEBFISHING_APPID);
    if webfishing.is_err() || webfishing.unwrap().is_none() {
        println!("Installing Webfishing");
        install_webfishing(&location).await;
    }

    let (app, library) = location.find_app(WEBFISHING_APPID).unwrap().unwrap();

    if !Path::exists("build/decompiler.zip".as_ref()) {
        download_gd_decomp().await;
    }

    if !Path::exists("build/godot_steam_template_324.zip".as_ref()) {
        download_godot_steam_template().await;
    }

    println!("Unzipping template 1/2");
    Command::new("unzip")
        .arg("-o")
        .arg("godot_steam_template_324.zip")
        .current_dir("./build")
        .output()
        .expect("Could not unzip godot_steam_template_324.zip");

    Command::new("mv")
        .arg("build/godot_steam_template_324/macos.zip")
        .arg("build/macos.zip")
        .current_dir("./build")
        .output()
        .expect("Could not copy godot_steam_template_324/macos.zip");

    println!("Unzipping template 2/2");
    Command::new("unzip")
        .arg("-o")
        .arg("macos.zip")
        .current_dir("./build")
        .output()
        .expect("Could not unzip macos.zip");

    let binding = library.resolve_app_dir(&app);
    let webfishing_path = binding.as_path();
    if !Path::exists(Path::new("build/webfishing.app")) {
        build_webfishing_macos(webfishing_path);
    }

    let _ = create_dir("build/webfishing-export").await;
    let mut bytes = vec![];
    File::open(webfishing_path.join("webfishing.exe"))
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut pck = PCK::from_bytes(&*bytes).unwrap();

    patches::steam_network_patch::patch(&mut pck).await;
    patches::options_menu_patch::patch(&mut pck).await;
    mods::mods::process_mods(&mut pck);

    let bytes = &pck.to_bytes();
    File::create("build/webfishing.app/Contents/Resources/webfishing.pck")
        .unwrap()
        .write(bytes)
        .expect("Could not write to webfishing.pck");

    Command::new("xattr")
        .arg("-cr")
        .arg("build/webfishing.app")
        .output()
        .expect("Could not execute xattr");

    println!("Webfishing is in the build folder !");

    Text::new("Press Enter to quit")
        .prompt()
        .expect("Could not confirm to quit");
}
