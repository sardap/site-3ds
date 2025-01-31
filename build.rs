// build.rs

use std::env;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use flate2::bufread::GzEncoder;
use flate2::write::DeflateEncoder;
use walkdir::WalkDir;

fn write_vec_to_contents(contents: &mut String, name: &str, data: &[u8]) {
    contents.push_str(&format!("pub const {}: [u8; {}] = [", name, data.len()));
    for byte in data {
        contents.push_str(&format!("{:#X}, ", byte));
    }
    contents.push_str("];\n");
}

type EncodingFunction = fn(&mut String, &str, &Path, usize) -> Option<(String, usize)>;

struct Encoder {
    field_name: &'static str,
    encoding_function: EncodingFunction,
}

impl Encoder {
    fn process(
        &self,
        compressed_size: &mut usize,
        mime_type: &str,
        data_section: &mut String,
        entries_section: &mut String,
        raw_name: &str,
        path: &Path,
        original_len: usize,
    ) {
        if mime_type.contains("video") || *compressed_size > original_len {
            entries_section.push_str(&format!("{}: None,\n", self.field_name));
        } else if let Some((data_name, size)) =
            (self.encoding_function)(data_section, raw_name, path, original_len)
        {
            entries_section.push_str(&format!("{}: Some(&{}),\n", self.field_name, data_name));
            *compressed_size += size;
        } else {
            entries_section.push_str(&format!("{}: None,\n", self.field_name));
        }
    }
}

fn gzip_encode_data(
    contents: &mut String,
    raw_name: &str,
    path: &Path,
    original_len: usize,
) -> Option<(String, usize)> {
    let data_gzip_name = format!("DATA_GZIP_{}", raw_name);
    let mut reader = BufReader::new(fs::File::open(path).unwrap());
    let mut encoder = GzEncoder::new(&mut reader, flate2::Compression::best());
    let mut gzip_data: Vec<u8> = vec![];
    encoder.read_to_end(&mut gzip_data).unwrap();
    let delta = original_len - gzip_data.len();
    if delta > original_len / 10 {
        write_vec_to_contents(contents, data_gzip_name.as_str(), &gzip_data);
        Some((data_gzip_name, gzip_data.len()))
    } else {
        None
    }
}

fn deflate_encode_data(
    contents: &mut String,
    raw_name: &str,
    path: &Path,
    original_len: usize,
) -> Option<(String, usize)> {
    let data = fs::read(path).unwrap();
    let data_deflate_name = format!("DATA_DEFLATE_{}", raw_name);
    let mut encoder = DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(&data).unwrap();
    let deflate_data: Vec<u8> = encoder.finish().unwrap();
    let delta = original_len - deflate_data.len();
    if delta > original_len / 10 {
        write_vec_to_contents(contents, &data_deflate_name, &deflate_data);
        Some((data_deflate_name, deflate_data.len()))
    } else {
        None
    }
}

fn br_encode_data(
    contents: &mut String,
    raw_name: &str,
    path: &Path,
    original_len: usize,
) -> Option<(String, usize)> {
    let data_br_name = format!("DATA_BR_{}", raw_name);
    let data = fs::read(path).unwrap();
    let mut encoder = brotli::CompressorReader::new(data.as_slice(), 4096, 11, 22);
    let mut br_data: Vec<u8> = vec![];
    encoder.read_to_end(&mut br_data).unwrap();
    let delta = original_len - br_data.len();
    if delta > original_len / 10 {
        write_vec_to_contents(contents, &data_br_name, &br_data);
        Some((data_br_name, br_data.len()))
    } else {
        None
    }
}

fn zstd_encode_data(
    contents: &mut String,
    raw_name: &str,
    path: &Path,
    original_len: usize,
) -> Option<(String, usize)> {
    let data_zstd_name = format!("DATA_ZSTD_{}", raw_name);
    let data = fs::read(path).unwrap();
    let mut encoder = zstd::stream::Encoder::new(Vec::new(), 22).unwrap();
    encoder.write_all(&data).unwrap();
    let zstd_data = encoder.finish().unwrap();
    let delta = original_len - zstd_data.len();
    if delta > original_len / 10 {
        write_vec_to_contents(contents, &data_zstd_name, &zstd_data);
        Some((data_zstd_name, zstd_data.len()))
    } else {
        None
    }
}

fn main() {
    let encoders: [Encoder; 4] = [
        Encoder {
            field_name: "body_br",
            encoding_function: br_encode_data,
        },
        Encoder {
            field_name: "body_gzip",
            encoding_function: gzip_encode_data,
        },
        Encoder {
            field_name: "body_deflate",
            encoding_function: deflate_encode_data,
        },
        Encoder {
            field_name: "body_zstd",
            encoding_function: zstd_encode_data,
        },
    ];

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
            pub body_br: Option<&'static [u8]>,
            pub body_zstd: Option<&'static [u8]>,
        }
        ",
    );

    let mut entries = vec![];

    let mut data_section = String::new();
    let empty_body_name = "EMPTY_BODY";
    write_vec_to_contents(&mut data_section, empty_body_name, &[]);

    let mut entries_section = String::new();

    for entry in WalkDir::new("site/dist") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let trimmed_path = path.strip_prefix("site/dist").unwrap();
        let raw = fs::read(path).unwrap();
        let raw_name = trimmed_path
            .to_str()
            .unwrap()
            .replace(".", "_")
            .replace("-", "_")
            .replace("/", "_")
            .to_uppercase();
        let get_var_name = format!("REQUEST_GET_{}", raw_name);
        let data_name = format!("DATA_{}", raw_name);
        write_vec_to_contents(&mut data_section, data_name.as_str(), &raw);

        let mime_type = mime_guess::from_path(path).first_or_octet_stream();

        entries_section.push_str(&format!(
            "pub const {}: ServeRequest = ServeRequest {{\n",
            get_var_name
        ));
        entries_section.push_str("method: \"GET\",\n");
        entries_section.push_str(&format!("path: \"/{}\",\n", trimmed_path.to_str().unwrap()));
        entries_section.push_str(&format!("content_type: \"{}\",\n", mime_type));
        entries_section.push_str(&format!("body: &{},\n", data_name));

        let mut compressed_data_size = 0;

        for encoder in encoders.iter() {
            encoder.process(
                &mut compressed_data_size,
                mime_type.essence_str(),
                &mut data_section,
                &mut entries_section,
                &raw_name,
                path,
                raw.len(),
            );
        }

        entries_section.push_str("};\n");

        entries.push(get_var_name);
    }

    contents.push_str(&data_section);
    contents.push_str(&entries_section);

    contents.push_str("pub const SERVE_REQUESTS: [ServeRequest; ");
    contents.push_str(&format!("{}] = [\n", entries.len()));
    for entry in entries {
        contents.push_str(&format!("    {},\n", entry));
    }
    contents.push_str("];\n");

    fs::write(&dest_path, contents).unwrap();
    println!("cargo::rerun-if-changed=site/public");
    println!("cargo::rerun-if-changed=site/src");
    println!("cargo::rerun-if-changed=site/package.json");
    println!("cargo::rerun-if-changed=site/vite.config.ts");
}
