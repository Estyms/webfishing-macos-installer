mod utils;
mod patches;

use std::fs::File;
use std::io::{Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use asky::Confirm;
use async_std::fs::create_dir;
use steamlocate::SteamDir;
use sudo::RunningAs;
use sysinfo::ProcessesToUpdate;

static WEBFISHING_APPID: u32 = 3146520;



async fn install_webfishing(location: &SteamDir) {
    let steam_location = location.path();
    let acf_path = steam_location.join("steamapps").join(format!("appmanifest_{}.acf", WEBFISHING_APPID));

    println!("Creating Webfishing ACF");
    File::create(acf_path).unwrap().write(include_str!("../res/webfishing.acf").as_bytes()).expect("could not write acf");

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
    let download_path = steam_location.join("steamapps").join("downloading").join(format!("{}", WEBFISHING_APPID));

    while Path::exists(download_path.as_path()) {
        println!("Downloading webfishing...");
        async_std::task::sleep(Duration::from_secs(10)).await;
    }
}

async fn download_godot_steam_template() {
    println!("Downloading GodotSteam template...");
    let res = reqwest::get("https://github.com/GodotSteam/GodotSteam/releases/download/v3.27/macos-g36-s160-gs327.zip").await.expect("Could not download godotsteam template");
    let body = res.bytes().await.expect("Could not read body");

    let mut file = File::create("build/godot_steam_template.zip").expect("Could not create godotsteam template");
    file.write_all(&body).expect("Could not write godotsteam template");
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
        .output().expect("Could not unzip godotsteam template");
}

fn build_webfishing_macos(webfishing_path: &Path) {
    let template_path = Path::new("build/osx_template.app");
    Command::new("rm")
        .current_dir(template_path)
        .arg("Contents/MacOS/godot_osx_debug.64")
        .output().expect("Could not remove delete godot_osx_debug.64");

    Command::new("mv")
        .current_dir(template_path)
        .arg("Contents/MacOS/godot_osx_release.64")
        .arg("Contents/MacOS/webfishing")
        .output().expect("Could not rename godot_osc_release.64");

    let mut steamappid = File::create(template_path.join("Contents").join("MacOS").join("steam_appid.txt")).expect("could not create steam_appid.txt file");
    steamappid.write(include_str!("../res/steam_appid.txt").as_bytes()).expect("could not write steam_appid.txt");

    Command::new("cp")
        .arg(webfishing_path.join("webfishing.exe"))
        .arg(template_path.join("Contents").join("Resources").join("webfishing.pck"))
        .output().expect("Could not copy webfishing.exe");

    let mut info_plist = File::create(template_path.join("Contents").join("Info.plist")).expect("Could not open Info.plist");
    info_plist.write_all(include_str!("../res/Info.plist").as_bytes()).expect("could not write Info.plist");

    Command::new("mv")
        .arg(template_path)
        .arg(Path::new("build/webfishing.app"))
        .output().expect("Could not copy webfishing.app");
}

fn decomp_game() {
    let webfishing_pck_path = Path::new("build").join("webfishing.app").join("Contents").join("Resources").join("webfishing.pck");
    let decomp_command = "build/Godot RE Tools.app/Contents/MacOS/Godot RE Tools";
    Command::new(decomp_command)
        .arg("--headless")
        .arg(format!("--extract={}", webfishing_pck_path.display()))
        .arg("--include=\"*options_menu*\"")
        .arg("--include=\"*SteamNetwork*\"")
        .arg("--output-dir=build/webfishing-export")
        .output().expect("Could not extract game");
}

#[tokio::main]
async fn main() {
    if !Path::exists("build".as_ref()) {
        println!("Creating build folder");
        create_dir("build").await.expect("could not create build folder");
    }

    let location = SteamDir::locate().expect("could not locate steam directory");

    let webfishing = location.find_app(WEBFISHING_APPID);
    if webfishing.is_err() || webfishing.unwrap().is_none() {
        println!("Installing Webfishing");
        install_webfishing(&location).await;
    }

    let (app, library) =  location.find_app(WEBFISHING_APPID).unwrap().unwrap();

    if !Path::exists("build/decompiler.zip".as_ref()) {
        download_gd_decomp().await;
    }

    if !Path::exists("build/godot_steam_template.zip".as_ref()) {
        download_godot_steam_template().await;
    }

    if !Path::exists("build/macos.zip".as_ref()) {
        println!("Unzipping template");
        Command::new("unzip")
            .arg("-o")
            .arg("godot_steam_template.zip")
            .current_dir("./build")
            .output().expect("Could not unzip godot_steam_template.zip");
    }

    if !Path::exists("build/osx_template.app".as_ref()) && !Path::exists("build/webfishing.app".as_ref()) {
        println!("Unzipping template");
        Command::new("unzip")
            .arg("-o")
            .arg("macos.zip")
            .current_dir("./build")
            .output()
            .expect("Could not unzip macos.zip");
    }


    let binding = library.resolve_app_dir(&app);
    let webfishing_path = binding.as_path();
    if !Path::exists(Path::new("build/webfishing.app")) {
        build_webfishing_macos(webfishing_path);
    }

    if sudo::check() != RunningAs::Root {
        decomp_game();
        patches::steam_network_patch::patch().await;
        patches::options_menu_patch::patch().await;
        println!("Root permissions needed to sign webfishing");
    }

    sudo::escalate_if_needed().expect("Could not escalate to sign the app");

    Command::new("xattr")
        .arg("-cr")
        .arg("build/webfishing.app")
        .output()
        .expect("Could not execute xattr");

    if Confirm::new("Do you wanna install Webfishing in the app folder?").prompt().expect("Could not confirm to install the webfishing") {
        Command::new("rsync")
            .arg("-a")
            .arg("build/webfishing.app")
            .current_dir("/Applications/")
            .output().expect("Could not execute rsync");

        Command::new("rm")
            .arg("-r")
            .arg("build/webfishing.app")
            .output().expect("Could not remove webfishing.app");

        println!("Successfully installed webfishing !");
    } else {
        println!("Webfishing is in the build folder !")
    }
}
