use std::{fs, thread::sleep_ms};

use hua_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir("hua")?;

    let mut store = Store::create_at_path("hua/store")?;
    println!("Store created successful");

    let mut user_manager = UserManager::create_at_path("hua/user")?;
    println!("User created successful");

    let package = Package::new("package", "0.1.0", "package");
    store.insert(package)?;
    println!("Package inserted successful");

    let hello_world = Package::new("hello-world", "1.0.0", "hello-world");
    store.insert(hello_world)?;

    let hello_world_hash = 2520265509;

    user_manager.insert_package(&hello_world_hash, &mut store)?;
    user_manager.remove_generation(0)?;
    user_manager.list_current_packages();

    fs::create_dir("global")?;
    let global_paths =
        ComponentPaths::new("global/bin", "global/etc", "global/lib", "global/share");
    global_paths.create_dirs()?;

    user_manager.link_current_global(global_paths)?;

    sleep_ms(5000);

    store.remove_unused(&user_manager)?;

    Ok(())
}
