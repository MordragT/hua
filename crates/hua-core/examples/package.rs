use std::{fs, thread::sleep_ms};

use hua_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir("hua")?;

    let mut store = Store::create_under("hua")?;
    println!("Store created successful");

    let mut user_manager = UserManager::create_under("hua")?;
    println!("User created successful");

    let package = Package::new("package", "0.1.0", "package");
    store.insert(package)?;
    println!("Package inserted successful");

    let hello_world = Package::new("hello-world", "1.0.0", "hello-world");
    store.insert(hello_world)?;

    let hash = 626824839214058140;

    // TODO link_package is broken
    // adding a package should increment the generation
    user_manager.insert_package(hash, &store)?;
    user_manager.remove_generation(0)?;
    user_manager.list_packages();

    fs::create_dir("global")?;
    let global_paths =
        ComponentPaths::new("global/bin", "global/etc", "global/lib", "global/share");
    global_paths.create_dirs()?;

    user_manager.link_current_global(global_paths)?;

    sleep_ms(2000);

    store.remove_unused(&user_manager)?;

    Ok(())
}
