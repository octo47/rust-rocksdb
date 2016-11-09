extern crate gcc;
extern crate num_cpus;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub fn link(name: &str, bundled: bool) {
    use std::env::var;
    let target = var("TARGET").unwrap();
    let target: Vec<_> = target.split('-').collect();
    if target.get(2) == Some(&"windows") {
        println!("cargo:rustc-link-lib=dylib={}", name);
        if bundled && target.get(3) == Some(&"gnu") {
            let dir = var("CARGO_MANIFEST_DIR").unwrap();
            println!("cargo:rustc-link-search=native={}/{}", dir, target[0]);
        }
    }
}

fn main() {

	  let mut snappy_config = gcc::Config::new();
	  snappy_config.include("snappy/");
	  snappy_config.include(".");

	  snappy_config.define("NDEBUG", Some("1"));

	  if !cfg!(target_env = "msvc") {
		    snappy_config.flag("-std=c++11");
	  } else {
		    snappy_config.flag("-EHsc");
	  }

	  snappy_config.file("snappy/snappy.cc");
	  snappy_config.file("snappy/snappy-sinksource.cc");
	  snappy_config.file("snappy/snappy-c.cc");
	  snappy_config.cpp(true);
	  snappy_config.compile("libsnappy.a");

    gcc::Config::new();
    let archives_dir = env::current_dir()
        .unwrap()
        .join("rocksdb/")
        .canonicalize()
        .unwrap();

    let cpu_num = num_cpus::get();
    assert!(process::Command::new("make")
            .env("DEBUG_LEVEL", "0")
            .env("SNAPPY", "1")
            .env("CFLAGS", "-I snappy/")
            .arg("-j").arg(format!("{}", cpu_num))
            .arg("static_lib")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .current_dir(&archives_dir)
            .output()
            .unwrap()
            .status
            .success());

    let pic_file = archives_dir.join("librocksdb.a");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let pic_file_dest = Path::new(&out_dir).join("librocksdb.a");
    fs::copy(pic_file, pic_file_dest).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());

}
