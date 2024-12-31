use std::fs;
use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use godot_pck::structs::{PckFile, PCK};
use crate::mods::structs::{Manifest, Patch, PckInfo};
use crate::utils;

fn read_manifests() -> Vec<Manifest> {
    let mut manifests = Vec::new();
    let mods_dir = fs::read_dir("mods").expect("Cannot read mods directory");
    for mod_dir in mods_dir {
        let mod_dir = mod_dir.expect("Failed to read mod directory");
        if !mod_dir.path().is_dir() {continue}

        if !mod_dir.path().join("manifest.json").exists() {
            println!("No manifest found in {:?}", mod_dir.path());
            continue;
        }

        let mut manifest: Manifest = serde_json::from_reader(fs::File::open(mod_dir.path().join("manifest.json")).unwrap()).unwrap();
        manifest.path = mod_dir.path().display().to_string();

        manifests.push(manifest);
    }
    manifests
}

fn check_mods(manifests: Vec<Manifest>) -> Vec<Manifest> {
    let mod_names: Vec<String> = manifests.iter().map(|m| m.get_name()).collect();

    manifests.into_iter().filter(|manifest: &Manifest| {
        let mut res = true;
        println!("Checking {}", manifest.get_name());
        for mod_name in manifest.get_dependencies() {
            if !mod_names.contains(&mod_name) {
                println!("\t- missing dependency {}", mod_name);
                res = false;
            }
        }
        res
    }).collect::<Vec<Manifest>>()
}

fn apply_patch(manifest: &Manifest, pck: &mut PCK, patch: &&Patch) {
    let resource_path = patch.get_resource();
    let resource_path = resource_path.as_str();
    let resource = pck.get_file_by_path_mut(resource_path).expect(&format!("Could not patch resource {}", resource_path));
    let resource_name = resource_path.split("res://").last().expect("could not remove res://").split("/").last().expect("Could not get last part of resource");
    if resource_path.ends_with(".gdc") {
        let mut resource_exported = File::create(format!("build/webfishing-export/{}", resource_name)).expect(&format!("Could not create mod resource {}", resource_name));
        resource_exported.write_all(resource.get_content()).expect(&format!("Could not write mod resource {}", resource_name));
        drop(resource_exported);
        utils::gd_utils::decomp_script(&format!("build/webfishing-export/{}", resource_name))
    } else {
        let mut resource_exported = File::create(format!("build/webfishing-decomp/{}", resource_name)).expect(&format!("Could not create mod resource {}", resource_name));
        resource_exported.write_all(resource.get_content()).expect(&format!("Could not write mod resource {}", resource_name));
        drop(resource_exported);
    }

    let mut patch_file = File::open(format!("{}/{}", manifest.path, patch.get_patch_file())).expect(&format!("Could not open patch file {}", patch.get_patch_file()));
    let mut patch_file_content = String::new();
    patch_file.read_to_string(&mut patch_file_content).expect(&format!("Could not read patch file {}", patch.get_patch_file()));
    drop(patch_file);

    let patch_struct = patch_apply::Patch::from_single(patch_file_content.as_str()).expect(&format!("Could not create patch {}", patch.get_patch_file()));

    let mut file_to_patch = File::open(format!("build/webfishing-decomp/{}", resource_name.replace(".gdc", ".gd"))).expect(&format!("Could not open file {}", resource_name));
    let mut file_to_patch_content = String::new();
    file_to_patch.read_to_string(&mut file_to_patch_content).expect(&format!("Could not read patch file {}", resource_name));
    drop(file_to_patch);

    let patched_content = patch_apply::apply(file_to_patch_content, patch_struct);

    if resource_path.ends_with(".gdc") {
        let mut patched_file = File::create(format!("build/webfishing-decomp/{}", resource_name.replace(".gdc", ".gd"))).unwrap();
        patched_file.write_all(patched_content.as_bytes()).unwrap();
        drop(patched_file);

        utils::gd_utils::recomp_file(&format!("build/webfishing-decomp/{}", resource_name.replace(".gdc", ".gd")));

        let mut recompiled_patched_file = File::open(format!("build/webfishing-recomp/{}", resource_name)).expect("Could not open recompiled patched file");
        let mut recompiled_patched_content = Vec::new();
        recompiled_patched_file.read_to_end(&mut recompiled_patched_content).expect(&format!("Could not read patched file {}", resource_name));

        resource.set_content(recompiled_patched_content);
    } else {
        resource.set_content(Vec::from(patched_content.as_bytes()));
    }
}

fn add_recursive_files_to_pck(dir: String, pck: &mut PCK, pck_info: &PckInfo) {
    let directory = read_dir(Path::new(dir.as_str())).expect("read_dir failed");
    for entry in directory {
        let entry = entry.expect("Failed to read entry");
        if entry.path().is_file() {
            let path: PathBuf = entry.path().iter()
                .skip_while(|s| *s != pck_info.get_directory().as_str())
                .skip(1)
                .collect();

            let resource_path = format!("{}/{}", pck_info.get_resource_prefix(), path.display()).replace("//", "/").replace("res:/", "res://");

            let mut resource_file = File::open(entry.path()).expect("Cannot open resource file");
            let mut resource_file_content = Vec::new();
            resource_file.read_to_end(&mut resource_file_content).expect("Cannot read resource file");
            drop(resource_file);

            let pck_file = PckFile::new_file(resource_path, resource_file_content);
            pck.add_file(pck_file);
        } else {
            add_recursive_files_to_pck(entry.path().display().to_string(), pck, pck_info);
        }
    }
}

fn apply_mod(manifest: &Manifest, pck: &mut PCK) {
    println!("Applying {} by {}", manifest.get_name(), manifest.get_author());
    // Apply game patches
    for patch in manifest.get_patches() {
        apply_patch(manifest, pck, &patch);
    };

    // Add pck data
    match manifest.get_pck_info() {
        Some(pck_info) => {
            add_recursive_files_to_pck(format!("{}/{}",manifest.path,pck_info.get_directory()), pck, pck_info);
        }
        None => {}
    }
}


pub fn process_mods(pck: &mut PCK) {
    if !Path::exists("mods".as_ref()) {
        fs::create_dir(Path::new("mods")).expect("Failed to create mods directory");
    }

    let manifests = read_manifests();
    println!("Found {} mods", manifests.len());

    // Dependency checking
    println!("Checking mod dependencies");
    let mut checked_manifests = check_mods(manifests);
    while !checked_manifests.clone().into_iter().map(|x| x.get_name()).eq(check_mods(checked_manifests.clone()).into_iter().map(|x| x.get_name())) {
        checked_manifests = check_mods(checked_manifests);
    }

    println!("Checked all mod dependencies, loading {} mods", checked_manifests.len());

    for manifest in checked_manifests {
        apply_mod(&manifest, pck)
    }
}