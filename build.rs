// build.rs

use std::env;
use std::fs;
use std::path::Path;

use flate2::write::DeflateEncoder;
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

fn write_vec_to_contents(contents: &mut String, name: &str, data: &[u8]) {
    contents.push_str(&format!("pub const {}: [u8; {}] = [", name, data.len()));
    for byte in data {
        contents.push_str(&format!("{:#X}, ", byte));
    }
    contents.push_str("];\n");
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
            pub body_deflate: Option<&'static [u8]>,
            pub body_gzip: Option<&'static [u8]>,
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
        write_vec_to_contents(&mut contents, data_name.as_str(), &raw);

        let data_gzip_name = {
            let data_gzip_name = format!("DATA_GZIP_{}", raw_name);
            let encoder = DeflateEncoder::new(raw.clone(), flate2::Compression::default());
            let gzip_data: Vec<u8> = encoder.finish().unwrap();
            if gzip_data.len() < raw.len() {
                write_vec_to_contents(&mut contents, data_gzip_name.as_str(), &gzip_data);
                Some(data_gzip_name)
            } else {
                None
            }
        };
        let data_deflate_name = {
            let data_deflate_name = format!("DATA_DEFLATE_{}", raw_name);
            let encoder = DeflateEncoder::new(raw.clone(), flate2::Compression::fast());
            let deflate_data: Vec<u8> = encoder.finish().unwrap();
            if deflate_data.len() < raw.len() {
                write_vec_to_contents(&mut contents, &data_deflate_name, &deflate_data);
                Some(data_deflate_name)
            } else {
                None
            }
        };

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
        if let Some(data_deflate_name) = data_deflate_name {
            contents.push_str(&format!("body_deflate: Some(&{}),\n", data_deflate_name));
        } else {
            contents.push_str("body_deflate: None,\n");
        }
        if let Some(data_gzip_name) = data_gzip_name {
            contents.push_str(&format!("body_gzip: Some(&{}),\n", data_gzip_name));
        } else {
            contents.push_str("body_gzip: None,\n");
        }
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
