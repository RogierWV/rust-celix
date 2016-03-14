extern crate rustc_serialize;
extern crate toml;
// use std::env::args;
use std::fs::{File,metadata,copy};
// use std::path::Path;
use std::io::prelude::*;
use std::process::Command;
use rustc_serialize::json::Json;

fn main() {
	println!("cargo-celix version {}", env!("CARGO_PKG_VERSION"));
	if !check_built(".so") {
		println!("Need to build!");
		println!("{}",cargo("build"));
	}
	copy_bundles("/usr/local/share/celix");
	create_bundle();
	write_config();
}

fn manifest() -> String {
	let package_name = toml_lookup("package.name").replace("-","_");
	return format!(
"Manifest-Version: 1.0
Bundle-SymbolicName: {}
Bundle-Name: {}
Bundle-Version: {}
Bundle-Activator: lib{}.so
Private-Library: lib{}.so
", package_name,package_name,toml_lookup("package.version"),package_name,package_name);
}

fn copy_bundles(celix_dir: &str) {
	let bundle_dir = celix_dir.to_string() + "/bundles/";
	let _ = Command::new("mkdir").arg("-p").arg("deploy/bundles").output().unwrap();
	copy(bundle_dir.as_str().clone().to_string() + "shell.zip", "deploy/bundles/shell.zip").unwrap();
	copy(bundle_dir.as_str().clone().to_string() + "shell_tui.zip", "deploy/bundles/shell_tui.zip").unwrap();
}

fn write_config(){
	let conf = format!("cosgi.auto.start.1=bundles/{}.zip bundles/shell_tui.zip bundles/shell.zip", toml_lookup("package.name").replace("-","_"));
	File::create("deploy/config.properties").unwrap().write_all(conf.as_bytes()).unwrap();
}

fn create_bundle() {
	let tmpdir : String = String::from_utf8_lossy(&Command::new("mktemp").arg("-dt").arg("rust-celix.XXXXXXXXXXXXXX").output().unwrap().stdout).into_owned().trim_right().to_string() + "/";
	let _ = copy(get_lib_name("target/release/lib",".so"),get_lib_name((tmpdir.as_str().clone().to_string()+"lib").as_str().clone(),".so"));
	let _ = Command::new("mkdir").arg("-p").arg(tmpdir.as_str().clone().to_string()+"META-INF").output().unwrap().stdout;
	File::create(tmpdir.as_str().clone().to_string()+"META-INF/MANIFEST.MF").unwrap().write_all(manifest().as_bytes()).unwrap();
	println!("{}", String::from_utf8_lossy(&Command::new("ls").current_dir(tmpdir.as_str().clone()).output().unwrap().stdout));
	println!("{}", String::from_utf8_lossy(&Command::new("zip").current_dir(tmpdir.as_str().clone()).arg(get_lib_name("",".zip")).arg(get_lib_name("lib",".so")).arg("META-INF/MANIFEST.MF").output().unwrap().stdout));
	let _ = copy(get_lib_name(tmpdir.as_str().clone(),".zip"),("deploy/bundles/".to_string()+toml_lookup("package.name").replace("-","_").as_str()).to_string()+".zip");
	let _ = Command::new("rm").arg("-rf").arg(tmpdir).output().unwrap().stdout;
}

fn check_built(ext: &str) -> bool {
	metadata(get_lib_name("target/release/lib",ext)).is_ok()
}

fn get_cargo_toml_path() -> String {
	let output = cargo("locate-project");
	let json = 
		match Json::from_str(&*output) {
			Ok(j) => j,
			Err(_) => panic!("Couldn't parse the output of `cargo locate-project`")
		};
	json["root"].as_string().unwrap().to_string()
}

fn cargo(command: &str) -> String {
	if command == "build" {
		let output = Command::new("cargo").arg("build").arg("--release").output().unwrap();
		return String::from_utf8_lossy(&output.stdout).into_owned();
	} else {
		let output = Command::new("cargo").arg(command).output().unwrap();
		return String::from_utf8_lossy(&output.stdout).into_owned();
	}
}

fn get_lib_name(pre: &str, ext: &str) -> String {
	let base_name = toml_lookup("package.name").replace("-","_");
	((pre.to_string() + base_name.as_str()) + ext).to_string()
}

fn toml_lookup(name: &str) -> String {
	let toml_path = get_cargo_toml_path();
	let mut toml_file = match File::open(toml_path) {
			Ok(f) => f,
			Err(_) => panic!("Couldn't open Cargo.toml")
		};
	
	let mut contents = String::new();
	toml_file.read_to_string(&mut contents).unwrap();

	let mut parser = toml::Parser::new(&*contents);
	let toml_table = toml::Value::Table(parser.parse().unwrap());

	toml_table.lookup(name).expect("Couldn't find the index!").as_str().unwrap().to_string()
}
