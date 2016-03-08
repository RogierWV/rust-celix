#![feature(type_ascription)]

extern crate bindgen;
extern crate cmacros;
use std::env;
use std::fs::{read_dir,File};
use std::io::prelude::*;
use std::path::{Path,MAIN_SEPARATOR};

fn gen_headers<'a>(path: &'a str, root: &'a str) -> (String,String) {
	let mut main_buf = String::new();
	let mut def_buf = String::new();
	for entry in read_dir(path).unwrap() {
		let f = entry.unwrap();
		let ftype = f.file_type().unwrap_or_else(|e|{
			panic!("Failed to get filetype of file {:?}: {}", f.path(), e);
		});
		if ftype.is_file() {
			main_buf.push_str("#include<"); 
			// let d: & str = f.path().strip_prefix(root).unwrap().parent().unwrap().to_str().unwrap();
			main_buf.push_str(f.path().strip_prefix(root).unwrap().parent().unwrap().to_str().unwrap());
			if f.path().strip_prefix(root).unwrap().parent().unwrap().to_str().unwrap() != "" {
				main_buf.push(MAIN_SEPARATOR);
			}
			main_buf.push_str(f.file_name().to_str().unwrap());
			main_buf.push_str(">\n");

			let mut header_src = String::new();
			File::open(f.path()).unwrap().read_to_string(&mut header_src).unwrap();
			let macros = cmacros::extract_macros(&header_src).unwrap();
			def_buf.push_str(cmacros::generate_rust_src(&macros, move |def| {
					if def.name == "true" || def.name == "false" || def.name == "bool" {
						return cmacros::TranslateAction::Skip;
					}
					match def.body {
						Some(ref s) => if s.as_str().contains("__attribute__") || s.as_str().contains("__declspec") || s.as_str().contains("PTHREAD_ONCE_INIT") {
								cmacros::TranslateAction::Skip
							} else {
								cmacros::translate_macro(def)
							},
						None => cmacros::translate_macro(def)
					}
				}
			).as_str());
		}
		else if ftype.is_dir() {
			let (tmp_main,tmp_def) = gen_headers(f.path().to_str().unwrap(), root);
			main_buf.push_str(tmp_main.as_str());
			def_buf.push_str(tmp_def.as_str());
		}
	}
	(main_buf,def_buf)
}

fn main() {
	let mut celix_path: String = String::from("/usr/local/include/celix");
	match env::var("CELIX_PATH") {
		Err(_) => (),
		Ok(ref s) => celix_path = s.clone(),
	}
	let (tmp_main,tmp_def) = gen_headers(celix_path.as_str(), celix_path.as_str());
	let mut f = File::create(Path::new(env::var("OUT_DIR").unwrap().as_str()).join("inc.h")).unwrap();
	f.write_all(tmp_main.as_bytes()).unwrap();

	let mut def = File::create(Path::new(env::var("OUT_DIR").unwrap().as_str()).join("constants.rs")).unwrap();
	def.write_all(tmp_def.as_bytes()).unwrap();
	
	let mut bindings = bindgen::builder();
	bindings.header(String::from(Path::new(env::var("OUT_DIR").unwrap().as_str()).join("inc.h").to_str().unwrap()));
	bindings.emit_builtins();
	bindings.clang_arg("-I");
	bindings.clang_arg(celix_path);
	let bindings = bindings.generate();
	let bindings = bindings.unwrap();
	bindings.write_to_file(Path::new(env::var("OUT_DIR").unwrap().as_str()).join("celix_bind.rs")).unwrap();
}
