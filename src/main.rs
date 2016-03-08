extern crate zip;
// use zip::{ZipWriter,CompressionMethod::*};
// use std::io::Write;
extern crate rustc_serialize;
extern crate toml;
use std::env::args;
use std::fs::{File,metadata};
// use std::io::Read;
use std::io::prelude::*;
use std::process::Command;
use rustc_serialize::json::Json;

fn main() {
	if !check_built(".so") {
		println!("Need to build!");
		println!("{}",cargo("build"));
		zip();
	}
	println!("{}", get_lib_name(".so"));

}

fn zip() {
	// let mut f = try!(File::create(get_lib_name(".zip")));
	// println!("Creating {}", f);
	// let mut zip = ZipWriter::new(f);
	// try!(zip.start_file("lib"+toml_lookup("package.name")+".zip", Stored));
	// try!(zip.write());

	println!("{}", String::from_utf8_lossy(&Command::new("zip").arg(get_lib_name(".zip")).arg(get_lib_name(".so")).output().unwrap().stdout));
}

fn check_built(ext: &str) -> bool {
	metadata(get_lib_name(ext)).is_ok()
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
		let output = Command::new("cargo")
							 .arg("build")
							 .arg("--release")
							 .output()
							 .unwrap();
		return String::from_utf8_lossy(&output.stdout).into_owned();
	} else {
		let output = Command::new("cargo")
							 .arg(command)
							 .output()
							 .unwrap();
		return String::from_utf8_lossy(&output.stdout).into_owned();
	}
}

fn get_lib_name(ext: &str) -> String {
	let base_name = toml_lookup("package.name").replace("-","_");
	(("target/release/lib".to_string() + base_name.as_str()) + ext).to_string()
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