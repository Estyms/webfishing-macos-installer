use std::process::Command;

const RE_TOOLS: &str = "build/Godot RE Tools.app/Contents/MacOS/Godot RE Tools";

pub(crate) fn decomp_script(path: &str) {
    Command::new(RE_TOOLS)
        .arg("--headless")
        .arg(format!("--decompile=\"{}\"", path))
        .arg("--bytecode=3.5.0")
        .arg("--output-dir=build/webfishing-decomp")
        .output().expect(format!("Failed to decompile file: {}", path).as_str());
}

pub(crate) fn recomp_file(path: &str) {
    Command::new(RE_TOOLS)
        .arg("--headless")
        .arg(format!("--compile=\"{}\"", path))
        .arg("--bytecode=3.5.0")
        .arg("--output-dir=build/webfishing-recomp")
        .output().expect(format!("Failed to recompile file: {}", path).as_str());
}