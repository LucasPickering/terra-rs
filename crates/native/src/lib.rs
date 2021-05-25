use std::{ffi::CString, os::raw::c_char};
use terra::{World, WorldConfig};

#[no_mangle]
pub extern "C" fn generate_world_json(radius: u16) -> *mut c_char {
    let config = WorldConfig {
        radius,
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    let json = world.to_json().unwrap();
    let cstring = CString::new(json).unwrap();
    cstring.into_raw()
}
