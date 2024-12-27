use std::process::Command;

const RE_TOOLS: &str = "build/Godot RE Tools.app/Contents/MacOS/Godot RE Tools";

// https://stackoverflow.com/a/54152901
pub(crate) fn replace_slice<T>(buf: &mut [T], from: &[T], to: &[T], replace_with: &[T])
where
    T: Clone + PartialEq + From<u8>,
{
    for i in 0..=buf.len() - replace_with.len() {
        if buf[i..].starts_with(from) {
            for j in (i + 1)..=buf.len()  {
                if buf[j..].starts_with(to) {
                    let mut vec = Vec::new();
                    vec.extend_from_slice(replace_with);
                    if replace_with.len() < j-i {
                        for _ in 0.. (j-i-replace_with.len()) {
                            vec.push(T::try_from(0).expect("Failed to convert from usize"));
                        }
                    }
                    buf[i..j].clone_from_slice(vec.as_slice());
                    break;
                }
            }
        }
    }
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