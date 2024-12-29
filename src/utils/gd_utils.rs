use std::process::Command;

const RE_TOOLS: &str = "build/Godot RE Tools.app/Contents/MacOS/Godot RE Tools";

// https://stackoverflow.com/a/54152901
pub(crate) fn replace_slice<T>(buf: &[T], from: &[T], to: &[T], replace_with: &mut [T]) -> Vec<T>
where
    T: Clone + PartialEq + From<u8>,
{
    let mut last_j = 0;
    let mut res : Vec<T> = Vec::new();
    for i in 0..=buf.len() {
        if buf[i..].starts_with(from) {
            res.append(&mut buf[last_j..i].to_vec());
            for j in (i + 1)..=buf.len()  {
                if buf[j..].starts_with(to) {
                    res.append(replace_with.to_vec().as_mut());
                    last_j = j;
                    break;
                }
            }
        }
    }

    res.append(&mut buf[last_j..].to_vec());
    res
}

pub(crate) fn decomp_file(path: &str) {
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