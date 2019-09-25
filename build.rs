extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("gl_bindings.rs");
    if dest.exists() {
        return;
    }
    
    let mut file = File::create(dest).unwrap();
    Registry::new(Api::Gl, (3, 2), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
