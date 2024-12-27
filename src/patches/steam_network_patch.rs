use async_std::fs::File;
use async_std::io::{ReadExt, WriteExt};
use crate::utils::gd_utils::replace_slice;

const SCRIPT_PATH: &str = "build/webfishing-decomp/SteamNetwork.gd";
const COMPILED_PATH: &str = "build/webfishing-recomp/SteamNetwork.gdc";
const GAME_PCK: &str = "build/webfishing.app/Contents/Resources/webfishing.pck";

pub(crate) async fn patch() {
    crate::utils::gd_utils::decomp_file("build/webfishing-export/Scenes/Singletons/SteamNetwork.gdc");

    let mut script = File::open(SCRIPT_PATH).await.expect("Cannot open script");
    let mut script_txt = String::new();
    script.read_to_string(&mut script_txt).await.expect("Cannot read script");
    drop(script);

    let patched_script = script_txt.replace("steam_id_remote", "remote_steam_id");
    let mut script = File::create(SCRIPT_PATH).await.expect("Cannot open script");
    script.write_all(patched_script.as_bytes()).await.expect("Cannot write");
    drop(script);

    crate::utils::gd_utils::recomp_file(SCRIPT_PATH);

    let mut compiled_script_bytes = Vec::new();
    let mut compiled_script = File::open(COMPILED_PATH).await.expect("Cannot open script");
    compiled_script.read_to_end(&mut compiled_script_bytes).await.expect("Cannot read");
    drop(compiled_script);

    let mut compiled_pck_bytes = Vec::new();
    let mut compiled_pck = File::open(GAME_PCK).await.expect("Cannot open pck");
    compiled_pck.read_to_end(&mut compiled_pck_bytes).await.expect("Cannot read");
    drop(compiled_pck);

    replace_slice(&mut compiled_pck_bytes,
                  &[0x47, 0x44, 0x53, 0x43, 0x0D, 0x00, 0x00, 0x00, 0x5B, 0x01, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00],
                  "GDSC".as_ref(),
                  &compiled_script_bytes
    );

    let mut compiled_pck = File::create(GAME_PCK).await.expect("Cannot open pck");
    compiled_pck.write_all(compiled_pck_bytes.as_slice()).await.expect("Cannot write");
}