use std::{error::Error, fs};

use clap::{arg, Command};
use hua_core::{extra::ComponentPaths, Package, Store, UserManager};

const HUA_PATH: &str = "hua";
const STORE_PATH: &str = "hua/store";
const USER_MANAGER_PATH: &str = "hua/user";
const GLOBAL_PATH: &str = "global";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("hua")
        .about("A simple package manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(vec![
            Command::new("init").about("Initialise the folder structure"),
            Command::new("store")
                .about("Do operations on the store")
                .arg_required_else_help(true)
                .subcommands(vec![
                    Command::new("search").about("Searches the store for the given path").arg(arg!(<NAME> "The name to search for")),
                    Command::new("collect-garbage").about("Collects all unused packages in the store and deletes them"),
                    ]),
            Command::new("generations")
                .about("Do operations on generations")
                .arg_required_else_help(true)
                .subcommands(vec![
                    Command::new("remove")
                        .about("Remove a specified generation")
                        .args(vec![
                            arg!(<ID> "The id of the generation to remove"),
                        ])
                        .arg_required_else_help(true),
                    Command::new("list").about("List all the generations of the current user")]
                ),
            Command::new("add")
                .about("Adds a package to the store and switches to a new generation with the package")
                .arg_required_else_help(true)
                .args(vec![arg!(<NAME> "The name of the package"), arg!(<VERSION> "The version of the package"), arg!(<PATH> "The path where the package is found")]),
            Command::new("remove")
                .about("Creates a new generation without the specified package and switches to the generation")
                .arg_required_else_help(true)
                .arg(arg!(<NAME> "The name of package"))
        ]).get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            fs::create_dir(HUA_PATH)?;
            fs::create_dir(GLOBAL_PATH)?;

            let global_paths = ComponentPaths::from_path(GLOBAL_PATH);
            global_paths.create_dirs()?;

            let _store = Store::create_at_path(STORE_PATH)?;
            let _user_manager = UserManager::create_at_path(USER_MANAGER_PATH)?;
            println!("Files and folders created!");
        }
        Some(("store", sub_matches)) => match sub_matches.subcommand() {
            Some(("search", sub_matches)) => {
                let name = sub_matches
                    .value_of("NAME")
                    .expect("When searching the store a package name has to be given.");
                let store = Store::open(STORE_PATH)?;
                store.search(name, |key, package| println!("Hash: {}: {}", key, package))?;
            }
            Some(("collect-garbage", _)) => {
                let mut store = Store::open(STORE_PATH)?;
                let user_manager = UserManager::open(USER_MANAGER_PATH)?;
                store.remove_unused(&user_manager)?;
                store.flush()?;
            }
            _ => unreachable!(),
        },
        Some(("generations", sub_matches)) => match sub_matches.subcommand() {
            Some(("list", _)) => {
                let user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.list_current_generations()?;
            }
            Some(("remove", sub_matches)) => {
                let id = sub_matches
                    .value_of("ID")
                    .expect("When removing a generation, a id has to be given.")
                    .parse()?;
                let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.remove_generation(id)?;

                user_manager.flush()?;
            }
            _ => unreachable!(),
        },
        Some(("add", sub_matches)) => {
            let name = sub_matches
                .value_of("NAME")
                .expect("When adding a package, a name has to be given.");
            let version = sub_matches
                .value_of("VERSION")
                .expect("When adding a package, a version has to be given.");
            let path = sub_matches
                .value_of("PATH")
                .expect("When adding a package, a path has to be provided.");

            let mut store = Store::open(STORE_PATH)?;
            let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;

            let package = Package::new(name, version, path);

            let hash = store.insert(package)?;
            user_manager.insert_package(&hash, &mut store)?;

            println!("success 1");

            let global_paths = ComponentPaths::from_path(GLOBAL_PATH);
            user_manager.link_current_global(global_paths)?;

            println!("success 2");

            store.flush()?;
            user_manager.flush()?;

            println!("success 3");
        }
        Some(("remove", sub_matches)) => {
            let name = sub_matches
                .value_of("NAME")
                .expect("When removing a package, a name has to be provided");

            let mut store = Store::open(STORE_PATH)?;
            let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;

            let hash = *store
                .search(name, |key, _package| *key)?
                .first()
                .expect(&format!("Package with the name {} not found", name));
            user_manager.remove_package(&hash, &mut store)?;

            let global_paths = ComponentPaths::from_path(GLOBAL_PATH);
            user_manager.link_current_global(global_paths)?;

            store.flush()?;
            user_manager.flush()?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
