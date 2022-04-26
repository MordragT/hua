#![feature(let_chains)]

use clap::{arg, Command};
use console::style;
use dialoguer::{Confirm, Select};
use hua_core::{
    cache::CacheBuilder,
    config::Config,
    extra::path::ComponentPathBuf,
    jail::{Bind, JailBuilder},
    recipe::{self, RecipeData},
    shell::ShellBuilder,
    store::{
        locator::{Locator, Source},
        package::Package,
        LocalStore, STORE_PATH,
    },
    url::Url,
    user::UserManager,
};
use std::{error::Error, fs, path::PathBuf};

const HUA_PATH: &str = "/hua";
const CONFIG_PATH: &str = "/hua/config.toml";
const USER_MANAGER_PATH: &str = "/hua/user";
const GLOBAL_PATH: &str = "/usr";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = Command::new("hua")
        .about("A simple package manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("init").about("Initialise the folder structure"),
            Command::new("store")
                .about("Do operations on the store")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("search").about("Searches the store for the given name").arg(arg!(<NAME> "The name to search for")),
                    Command::new("collect-garbage").about("Collects all unused packages in the store and deletes them"),
                    Command::new("add").about("Add package to the store").args([arg!(<LOCK_FILE> "The lock file of the package"), arg!(<PATH> "The path of the package files")])
                    ]),
            Command::new("generations")
                .about("Do operations on generations")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("remove")
                        .about("Remove a specified generation")
                        .arg(arg!(<ID> "The id of the generation to remove"))
                        .arg_required_else_help(true),
                    Command::new("list").about("List all the generations of the current user")]
                ),
            Command::new("add")
                .about("Adds a package to the store and switches to a new generation with the package")
                .arg_required_else_help(true)
                .arg(arg!(<NAME> "The name of the package")),
            Command::new("remove")
                .about("Creates a new generation without the specified package and switches to the generation")
                .arg_required_else_help(true)
                .arg(arg!(<NAME> "The name of package")),
            Command::new("build")
                .about("Builds a recipe to a new package")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> "The path to the recipe")),
            Command::new("shell")
                .about("Create a new shell with the specified packages in scope")
                .arg_required_else_help(true)
                .arg(arg!(<NAME> ... "The names of the packages to include in scope")),
            Command::new("cache").about("Change caches").arg_required_else_help(true).subcommands([
                Command::new("add").about("Adds a cache").arg(arg!(<URL> "The url of the cache")),
                Command::new("remove").about("Removes a cache"),
            ])
        ]).get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            let path = PathBuf::from(HUA_PATH);
            if !path.exists() {
                fs::create_dir(HUA_PATH)?;
            }

            let path = PathBuf::from(GLOBAL_PATH);
            if !path.exists() {
                fs::create_dir(path)?;
            }

            let global_paths = ComponentPathBuf::from_path(GLOBAL_PATH);
            global_paths.create_dirs()?;

            let _store = LocalStore::init(STORE_PATH)?;
            let _user_manager = UserManager::init(USER_MANAGER_PATH)?;
            let _config = Config::init(CONFIG_PATH, Vec::new())?;

            println!("Files and folders created");
        }
        Some(("store", sub_matches)) => match sub_matches.subcommand() {
            Some(("search", sub_matches)) => {
                let name = sub_matches
                    .value_of("NAME")
                    .expect("When searching the store a package name has to be given.");
                let store = LocalStore::open(STORE_PATH)?;
                for (id, desc, _objects) in store.packages().filter_by_name_containing(name) {
                    println!("{} {desc}\n", style(id.truncate()).blue());
                }
            }
            Some(("collect-garbage", _)) => {
                let mut store = LocalStore::open(STORE_PATH)?;
                let user_manager = UserManager::open(USER_MANAGER_PATH)?;
                let _removed = store.remove_unused(&user_manager)?;
                store.flush()?;
            }
            Some(("add", sub_matches)) => {
                let lock_file = sub_matches
                    .value_of("LOCK_FILE")
                    .expect("A lock file has to be provided when adding a package to the store");
                let path = sub_matches
                    .value_of("PATH")
                    .expect("A path has to be provided when adding a package to the store");

                let path = {
                    let path = PathBuf::from(path);
                    if path.is_symlink() {
                        path.read_link()?
                    } else {
                        path
                    }
                };

                let lock_data = fs::read(lock_file)?;
                let package = toml::from_slice::<Package>(&lock_data)?;

                let mut store = LocalStore::open(STORE_PATH)?;

                println!("Package to add:\n{}", &package.desc);
                if Confirm::new().with_prompt("Continue?").interact()? {
                    let name = package.name().clone();

                    store.insert(package, path)?;
                    store.flush()?;
                    println!("{} {name} added", style("Success").green());
                } else {
                    println!("Nothing added");
                }
            }
            _ => unreachable!(),
        },
        Some(("generations", sub_matches)) => match sub_matches.subcommand() {
            Some(("list", _)) => {
                let user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.list_current_generations();
            }
            Some(("remove", sub_matches)) => {
                let id = sub_matches
                    .value_of("ID")
                    .expect("When removing a generation, a id has to be given.")
                    .parse()?;
                let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.remove_generation(id)?;
                user_manager.flush()?;

                println!("{} {id} removed", style("Success").green());
            }
            _ => unreachable!(),
        },
        Some(("add", sub_matches)) => {
            // TODO should give a selection of packages found in the local store or remote caches and let the user decide

            let name = sub_matches
                .value_of("NAME")
                .expect("When adding a package, a name has to be given.");

            let mut store = LocalStore::open(STORE_PATH)?;
            let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;
            let config = Config::open(CONFIG_PATH)?;
            let locator = Locator::new(config.to_caches().into_iter())?;

            // let (names, packages): (Vec<_>, Vec<_>) = store
            //     .packages()
            //     .filter_by_name_containing(name)
            //     .map(|(id, desc, objects)| {
            //         (
            //             format!("{} {}", style(&desc.name).green(), desc.version),
            //             (id, desc, objects),
            //         )
            //     })
            //     .unzip();
            let (mut names, mut packages): (Vec<_>, Vec<_>) = locator
                .filter_by_name_containing(name, &store)
                .map(|(id, desc, source)| {
                    (
                        match &source {
                            Source::Local => {
                                format!("{} {}", style(&desc.name).green(), desc.version)
                            }
                            Source::Remote(url) => {
                                format!("{} {} {url}", style(&desc.name).green(), desc.version)
                            }
                        },
                        (id, desc, source),
                    )
                })
                .unzip();

            let selection = Select::new()
                .with_prompt("Wich package to add (cancel with ESC or q)?")
                .items(&names)
                .interact_opt()?;

            if let Some(index) = selection {
                let name = names.remove(index);
                let (id, desc, source) = packages.remove(index);
                let id = *id;
                let desc = desc.clone();

                if let Source::Remote(url) = source && !store.packages().contains(&id) {
                    let cache = CacheBuilder::default().build()?;
                    let path = cache.cached_path(url.as_str())?;
                    store.insert(Package::new(id, desc.clone()), path)?;
                }

                let blobs = unsafe { store.get_blobs_cloned_of_package(&id).unwrap_unchecked() };
                let req = (desc, blobs.collect()).into();

                user_manager.insert_requirement(req, &store)?;

                let global_paths = ComponentPathBuf::from_path(GLOBAL_PATH);
                user_manager.switch_global_links(&global_paths)?;

                user_manager.flush()?;

                println!("{} {name} added", style("Success").green());
            } else {
                println!("Nothing added");
            }
        }
        Some(("remove", sub_matches)) => {
            let name = sub_matches
                .value_of("NAME")
                .expect("When removing a package, a name has to be provided");

            let store = LocalStore::open(STORE_PATH)?;
            let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;

            let (names, reqs): (Vec<_>, Vec<_>) = user_manager
                .filter_requirements_by_name_containing(name)
                .map(|req| {
                    (
                        format!("{} {}", style(req.name()).green(), req.version_req()),
                        req,
                    )
                })
                .unzip();

            let selection = Select::new()
                .with_prompt("Wich package to remove (cancel with ESC or q)?")
                .items(&names)
                .interact_opt()?;

            if let Some(selection) = selection {
                let name = &names[selection];
                let req = reqs[selection].clone();

                user_manager.remove_requirement(&req, &store)?;

                let global_paths = ComponentPathBuf::from_path(GLOBAL_PATH);
                user_manager.switch_global_links(&global_paths)?;

                user_manager.flush()?;
                println!("{} {name} removed", style("Success").green());
            } else {
                println!("Nothing removed");
            }
        }
        Some(("build", sub_matches)) => {
            let path = sub_matches
                .value_of("PATH")
                .expect("A recipe has to be provided.");
            let data = fs::read(path)?;
            let recipe_data = toml::from_slice::<RecipeData>(&data)?;

            let store = LocalStore::open(STORE_PATH)?;
            let cache = CacheBuilder::default().build()?;

            let (package, path) = recipe::build_recipe(recipe_data, &store, &cache)?;

            println!("{} {path:#?}\n{package}", style("Success").green());
        }
        Some(("shell", sub_matches)) => {
            let names = sub_matches
                .values_of("NAME")
                .expect("When creating a shell, package names must be provided.");
            let cwd = std::env::current_dir()?;

            let store = LocalStore::open(STORE_PATH)?;

            let jail = JailBuilder::new()
                .bind(Bind::read_write(&cwd, &cwd))
                .current_dir(cwd);

            let shell = ShellBuilder::new()?.with_names(names, &store)?;
            let jail = shell.apply(jail)?;

            let mut child = jail.arg("sh").run()?;
            child.wait()?;
        }
        Some(("cache", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches)) => {
                let url = sub_matches
                    .value_of("URL")
                    .expect("When adding a cache a url has to be provided");
                let url = Url::parse(url)?;

                let mut config = Config::open(CONFIG_PATH)?;
                config.add_cache(url.clone());

                println!("{} {url} added", style("Success").green());
            }
            Some(("remove", _)) => {
                let mut config = Config::open(CONFIG_PATH)?;

                let selection = Select::new()
                    .with_prompt("Wich cache to remove (cancel with ESC or q)?")
                    .items(config.caches())
                    .interact_opt()?;

                if let Some(index) = selection {
                    let removed = config.remove_cache(index);
                    println!("{} {removed} removed", style("Success").green());
                } else {
                    println!("Nothing removed");
                }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(())
}
