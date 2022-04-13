#![allow(non_snake_case)]

use core::ffi::c_void;
use hua_core::{CacheBuilder, LocalBackend, Recipe, Store, Version, STORE_PATH};
use roc::{RocRecipe, RocRequirement};
use std::ffi::CStr;
use std::os::raw::c_char;

mod roc;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed"]
    fn roc_recipe() -> RocRecipe;
}

#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, _alignment: u32) -> *mut c_void {
    return libc::malloc(size);
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    c_ptr: *mut c_void,
    new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    return libc::realloc(c_ptr, new_size);
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(c_ptr: *mut c_void, _alignment: u32) {
    return libc::free(c_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    match tag_id {
        0 => {
            let slice = CStr::from_ptr(c_ptr as *const c_char);
            let string = slice.to_str().unwrap();
            eprintln!("Roc hit a panic: {}", string);
            std::process::exit(1);
        }
        _ => todo!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_memcpy(dst: *mut c_void, src: *mut c_void, n: usize) -> *mut c_void {
    libc::memcpy(dst, src, n)
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    libc::memset(dst, c, n)
}

#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
    let RocRecipe {
        name,
        version,
        desc,
        archs,
        platforms,
        // TODO checksum
        source,
        licenses,
        requires,
        requiresBuild,
        targetDir,
        envs,
        script,
    } = unsafe { roc_recipe() };

    let recipe = Recipe::new(
        name.to_string(),
        Version::parse(version.as_str()).unwrap(),
        desc.to_string(),
        archs,
        platforms,
        source.to_string(),
        licenses.into_iter().map(|l| l.to_string()).collect(),
        requires.into_iter().map(RocRequirement::into).collect(),
        requiresBuild
            .into_iter()
            .map(RocRequirement::into)
            .collect(),
        targetDir.to_string().into(),
    );

    let store = Store::<LocalBackend>::open(STORE_PATH).unwrap();
    let cache = CacheBuilder::new().build().unwrap();

    let (_package, _path) = recipe
        .fetch(&cache)
        .unwrap()
        .prepare_requirements(
            &store,
            envs.into_iter()
                .map(|tuple| (tuple.left.to_string(), tuple.right.to_string())),
        )
        .unwrap()
        .build(script.to_string())
        .unwrap();

    println!("Package created successfully!");

    // Exit code
    0
}
