#![feature(const_fn)]

extern crate toml;
extern crate getopts;
use getopts::Options;
use std::fs::{File,metadata,copy,create_dir_all};
use std::io::prelude::*;
use std::env;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::sync::{Once, ONCE_INIT};
use std::time::Instant;

static START: Once = ONCE_INIT;
static mut toml_path: *mut String = std::ptr::null_mut();

/// Expands into the expected `Command::new($command).current_dir($dir).arg($arg).arg($arg)...output().unwrap()`
macro_rules! cmd {
	( $command:expr, $dir:expr $(, $arg:expr )* ) => {
		match Command::new($command)
		.current_dir($dir)
		$(
			.arg($arg)
		)*
		.output()
		{
			Ok(output) => output,
			Err(e) => panic!("Failed to execute command {}: {}", $command, e)
		}
	};
}

macro_rules! write_file {
	( $filename:expr, $contents:expr ) => {
		File::create($filename)
		.unwrap()
		.write_all($contents)
		.expect("Failed to write to file!")
		// {
		// 	Ok(()) => println!("Succesfully written {}", $filename),
		// 	Err(e) => panic!("Failed to write to {}: {}", $filename, e)
		// }
	};
}

fn main() {
	println!("cargo-celix version {}", env!("CARGO_PKG_VERSION"));
	let args : Vec<String> = env::args().collect();
	// for arg in args {
	// loop {
	// 	match args.next() { None => break, Some(arg) => {
	//     println!("{} ", arg);
	// 	match arg.as_str() {
	// 		"new" => {
	// 			println!("{:?}", args.next());
	// 		},
	// 		"build" => {},
	// 		"--bin" => {},
	// 		"--release" => {}
	// 		_ => {}
	// 	}}}
	// }

	// let program = args[0].clone() + args[1].clone().as_str();
	let program = vec![args[0].clone(), args[1].clone()].join(" ");
	println!("{:?}", program);
	START.call_once(|| {
		unsafe {toml_path = Box::into_raw(Box::new(get_cargo_toml_path())); }
	});
	let _ = env::set_current_dir(Path::new(get_cargo_toml_path().as_str()).parent().unwrap());
	if !check_built(".so") {
		println!("Need to build!");
		// cmd!("cargo", ".", "build", "--release");
		let _ = Command::new("cargo").arg("build").arg("--release").status().unwrap();
	}
	let celix_dir = match env::var("CELIX_PATH") {
		Err(_) => String::from("/usr/local/"),
		Ok(ref s) => s.clone()
	};
	copy_bundles(celix_dir.as_str());
	create_bundle();
	write_config();
	println!("Done!");
	unsafe{
		let _ = Box::from_raw(toml_path);
	}
}

/// Generate MANIFEST.MF
///
/// Formats a base string using package name and version
fn manifest() -> String {
	let package_name = toml_lookup("package.name").replace("-","_");
	return format!(
"Manifest-Version: 1.0
Bundle-SymbolicName: {}
Bundle-Name: {}
Bundle-Version: {}
Bundle-Activator: lib{}.so
Private-Library: lib{}.so
CREATION-TIME: {:?}
", package_name,package_name,toml_lookup("package.version"),package_name,package_name,Instant::now());
}

/// Copy base Celix bundles
///
/// Copies the base Celix bundles to ./deploy/bundles
fn copy_bundles(celix_dir: &str) {
	let _ = env::set_current_dir(Path::new(get_cargo_toml_path().as_str()).parent().unwrap());
	let bundle_dir = celix_dir.to_string() + "/share/celix/bundles/";
	// let _ = Command::new("mkdir").arg("-p").arg("deploy/bundles").output().unwrap();
	// let _ = cmd!("mkdir", ".", "-p", "target/deploy/bundles");
	create_dir_all("target/deploy/bundles").expect("Failed to create `target/deploy/bundles`!");
	let bdir1 = bundle_dir.clone();
	let t1 = thread::spawn(move || {copy(bdir1.as_str().clone().to_string() + "shell.zip", "target/deploy/bundles/shell.zip").expect("Failed to copy shell.zip!")});
	let bdir2 = bundle_dir.clone();
	let t2 = thread::spawn(move || {copy(bdir2.as_str().clone().to_string() + "shell_tui.zip", "target/deploy/bundles/shell_tui.zip").expect("Failed to copy shell_tui.zip!")});
	let _ = t1.join();
	let _ = t2.join();
}

/// Writes config file
///
/// Uses a basic template (which starts this bundle along with shell and shell_tui) to generate and write a config file to ./deploy
fn write_config(){
	let _ = env::set_current_dir(Path::new(get_cargo_toml_path().as_str()).parent().unwrap());
	let conf =
		format!("cosgi.auto.start.1=bundles/{}.zip bundles/shell_tui.zip bundles/shell.zip",
			toml_lookup("package.name")
			.replace("-","_"));

	write_file!("target/deploy/config.properties", conf.as_bytes());
}

/// Create a Celix bundle from the built .so file.
///
/// Creates a temporary directory in /tmp, then moves the .so file there, creates a MANIFEST.MF there, zips them both, andcopies the result into ./deploy/bundles
fn create_bundle() {
	let _ = env::set_current_dir(Path::new(get_cargo_toml_path().as_str()).parent().unwrap());
	let tmpdir = String::from_utf8_lossy(
		&cmd!("mktemp", ".", "-dt", "rust-celix.XXXXXXXXXXXXXX").stdout)
		.into_owned()
		.trim_right()
		.to_string()
		+ "/";

	let _ =
		copy(
			get_lib_name(
				"target/release/lib",
				".so"),
			get_lib_name(
				(tmpdir
					.as_str()
					.clone()
					.to_string()
					+"lib")
				.as_str()
				.clone(),
				".so"));

	// let _ = cmd!("mkdir", ".", "-p", tmpdir.as_str().clone().to_string()+"META-INF");
	create_dir_all(tmpdir.as_str().clone().to_string()+"META-INF").expect("Failed to create META-INF in temp directory!");

	write_file!(tmpdir.as_str().clone().to_string()+"META-INF/MANIFEST.MF", manifest().as_bytes());

	// println!("{}",
	// 	String::from_utf8_lossy(
	// 		&cmd!( "tree", tmpdir.as_str().clone() ).stdout)
	// 	);

	&cmd!("jar",
		tmpdir.as_str().clone(),
		"cfm",
		get_lib_name("",".zip"),
		"META-INF/MANIFEST.MF",
		get_lib_name("lib",".so")
	);

	let _ =
		copy(
			get_lib_name(tmpdir.as_str().clone(),".zip"),
			("target/deploy/bundles/".to_string()
				+toml_lookup("package.name")
				.replace("-","_")
				.as_str())
			.to_string()
			+".zip");

	thread::spawn(||{let _ = cmd!("rm", ".", "-rf", tmpdir);});
}

/// Checks whether the .so exists
fn check_built(ext: &str) -> bool {
	metadata(get_lib_name("target/release/lib",ext)).is_ok()
}

/// Returns path to Cargo.toml
fn get_cargo_toml_path() -> String {
	String::from_utf8(
		Command::new("cargo")
		.arg("locate-project")
		.output()
		.expect("Failed to execute cargo locate-project")
		.stdout
	).unwrap()
	.to_owned()
	.split(':')
	.collect::<Vec<&str>>()[1]
	.replace("\"","")
	.replace("}","")
	.replace("\n","")
}

/// Gets the library name from the Cargo file
///
/// Prepends `pre` and appends `ext`.
/// Example:
/// ```
/// get_lib_name("lib",".so");
/// ```
/// Would return something along the lines of `libpackagename.so`
fn get_lib_name(pre: &str, ext: &str) -> String {
	let base_name = toml_lookup("package.name").replace("-","_");
	((pre.to_string() + base_name.as_str()) + ext).to_string()
}

/// Looks up the specified key in Cargo.toml
fn toml_lookup(name: &str) -> String {
	// let toml_path = get_cargo_toml_path();
	let mut toml_file;
	unsafe {
		// println!("toml_path: {:?}", &**toml_path);
		toml_file = match File::open(&*toml_path) {
				Ok(f) => f,
				Err(_) => panic!("Couldn't open Cargo.toml")
			};
		}

	let mut contents = String::new();
	toml_file.read_to_string(&mut contents).unwrap();

	let mut parser = toml::Parser::new(&*contents);
	let toml_table = toml::Value::Table(parser.parse().unwrap());

	toml_table.lookup(name).expect("Couldn't find the index!").as_str().unwrap().to_string()
}
