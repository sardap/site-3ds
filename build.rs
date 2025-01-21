// build.rs

use std::env;
use std::fs;
use std::path::Path;

use walkdir::WalkDir;

fn file_ext_to_content_type(ext: &str) -> &'static str {
    match ext {
        "txt" => "text/plain",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "ico" => "image/x-icon",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "text/plain",
    }
}

fn main() {
    // Compile the vue
    std::process::Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("site")
        .status()
        .expect("Failed to compile vue");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("dist.rs");
    // Loop over files in dist directory
    let mut contents = String::new();

    contents.push_str(
        "pub struct ServeRequest {
            pub method: &'static str,
            pub path: &'static str,
            pub content_type: &'static str,
            pub body: &'static [u8],
        }
        ",
    );

    let mut entries = vec![];

    for entry in WalkDir::new("site/dist") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let trimmed_path = path.strip_prefix("site/dist").unwrap();
        let raw = fs::read(path).unwrap();
        let ext = path.extension().unwrap().to_str().unwrap();
        let raw_name = trimmed_path
            .to_str()
            .unwrap()
            .replace(".", "_")
            .replace("/", "_")
            .to_uppercase();
        let var_name = format!("REQUEST_{}", raw_name);
        let data_name = format!("DATA_{}", raw_name);
        contents.push_str(&format!("pub const {}: [u8; {}] = [", data_name, raw.len()));
        for byte in raw {
            contents.push_str(&format!("{:#X}, ", byte));
        }
        contents.push_str("];\n");
        contents.push_str(&format!(
            "pub const {}: ServeRequest = ServeRequest {{\n",
            var_name
        ));
        contents.push_str("method: \"GET\",\n");
        contents.push_str(&format!("path: \"/{}\",\n", trimmed_path.to_str().unwrap()));
        contents.push_str(&format!(
            "content_type: \"{}\",\n",
            file_ext_to_content_type(ext)
        ));
        contents.push_str(&format!("body: &{},\n", data_name));
        contents.push_str("};\n");

        entries.push(var_name);
    }

    contents.push_str("pub const SERVE_REQUESTS: [ServeRequest; ");
    contents.push_str(&format!("{}] = [\n", entries.len()));
    for entry in entries {
        contents.push_str(&format!("    {},\n", entry));
    }
    contents.push_str("];\n");

    fs::write(&dest_path, contents).unwrap();
    println!("cargo::rerun-if-changed=site");
}
