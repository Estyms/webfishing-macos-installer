use async_std::fs::File;
use async_std::io::{ReadExt, WriteExt};
use godot_pck::structs::PCK;

const RESOURCE_PATH: &str = "res://Scenes/Singletons/OptionsMenu/options_menu.gdc";
const FILE_PATH: &str = "build/webfishing-export/options_menu.gdc";
const SCRIPT_PATH: &str = "build/webfishing-decomp/options_menu.gd";
const COMPILED_PATH: &str = "build/webfishing-recomp/options_menu.gdc";
pub(crate) async fn patch(pck: &mut PCK) {
    println!("Patching {} files...", RESOURCE_PATH);
    let pck_file = pck.get_file_by_path_mut(RESOURCE_PATH).expect("Couldn't find options_menu.gdc file");

    let content = pck_file.get_content();
    let mut exported_file = File::create(FILE_PATH).await.expect("Couldn't create file");
    exported_file.write_all(content).await.unwrap();
    drop(exported_file);

    crate::utils::gd_utils::decomp_script(FILE_PATH);

    let mut script = File::open(SCRIPT_PATH).await.expect("Cannot open script");
    let mut script_txt = String::new();
    script.read_to_string(&mut script_txt).await.expect("Cannot read script");
    drop(script);

    let patched_script = script_txt.replace("OS.window_borderless = PlayerData.player_options.fullscreen == 1", "");
    let mut script = File::create(SCRIPT_PATH).await.expect("Cannot open script");
    script.write_all(patched_script.as_bytes()).await.expect("Cannot write");
    drop(script);

    crate::utils::gd_utils::recomp_file(SCRIPT_PATH);

    let mut file = File::open(COMPILED_PATH).await.expect("Cannot open compiled script");
    let mut new_content = vec![];
    file.read_to_end(&mut new_content).await.unwrap();

    pck_file.set_content(new_content);
}