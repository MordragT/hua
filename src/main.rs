#![feature(let_chains)]
#![feature(unix_chown)]

use caps::{CapSet, Capability};
use clap::{arg, Command};
use console::style;
use dialoguer::Select;
use hua_core::{
    cache::CacheBuilder,
    config::Config,
    extra::path::ComponentPathBuf,
    jail::{Bind, JailBuilder},
    recipe::{self, build_recipe, Derivation},
    shell::ShellBuilder,
    store::{
        locator::Locator,
        package::{LocalPackageSource, RemotePackageSource},
        LocalStore, STORE_PATH,
    },
    url::Url,
    user::UserManager,
    GID, HUA_PATH, UID,
};
use log::{debug, info};
use std::{error::Error, fs, os::unix, path::PathBuf};

const CONFIG_PATH: &str = "/hua/config.toml";
const USER_MANAGER_PATH: &str = "/hua/user";
const REMOTE_TMP: &str = "/tmp/remote";

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
                    // Command::new("add").about("Add package to the store").args([arg!(<LOCK_FILE> "The lock file of the package"), arg!(<PATH> "The path of the package files")])
                    ]),
            Command::new("generations")
                .about("Do operations on generations")
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("remove")
                        .about("Remove a specified generation")
                        .arg(arg!(<ID> "The id of the generation to remove"))
                        .arg_required_else_help(true),
                    Command::new("list").about("List all the generations of the current user"),
                    Command::new("current").about("Returns the id of the current generation in use by the user"),
                    Command::new("switch")
                        .about("Switch to a specific generation")
                        .arg(arg!(<ID> "The id of the generation to switch to"))
                        .arg_required_else_help(true)]
                ),
            Command::new("add")
                .about("Adds a package to the store if not already existing and switches to a new generation with the package")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> "The path to the recipe")),
            Command::new("remove")
                .about("Creates a new generation without the specified package and switches to the generation")
                .arg_required_else_help(true)
                .arg(arg!(<NAME> "The name of package")),
            // TODO search
            Command::new("build")
                .about("Builds a recipe to a new package")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> "The path to the recipe")),
            Command::new("shell")
                .about("Create a new shell with the specified packages in scope")
                .arg_required_else_help(true)
                .arg(arg!(<NAME> ... "The names of the packages to include in scope")),
            Command::new("cache").about("Change caches").arg_required_else_help(true).subcommands([
                // TODO add list
                Command::new("add").about("Adds a cache").arg(arg!(<URL> "The url of the cache")),
                Command::new("remove").about("Removes a cache"),
            ])
        ]).get_matches();

    // TODO capdacoverride is sufficent no nead for read_search

    match matches.subcommand() {
        Some(("init", _)) => {
            if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
                && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
            {
                caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
            } else {
                return Err(
                    "Please run hua init as root or with the appropiate capabilities".into(),
                );
            }

            let path = PathBuf::from(HUA_PATH);
            if !path.exists() {
                fs::create_dir(&path)?;
                unix::fs::chown(path, UID, GID)?;
            }
            debug!("{HUA_PATH} created");

            // let path = PathBuf::from(GLOBAL_PATH);
            // if !path.exists() {
            //     fs::create_dir(&path)?;
            //     unix::fs::chown(path, UID, GID)?;
            // }
            // debug!("{GLOBAL_PATH} created");

            let global_paths = ComponentPathBuf::global();
            global_paths.create_dirs(true)?;
            debug!("Global component paths created");

            let _store = LocalStore::init(STORE_PATH)?;
            info!("Local store in {STORE_PATH} initialised");

            let _user_manager = UserManager::init(USER_MANAGER_PATH)?;
            info!("User manager in {USER_MANAGER_PATH} initialised");

            let _config = Config::init(CONFIG_PATH, Vec::new())?;
            info!("Config in {CONFIG_PATH} initialised");

            println!("Files and folders created");
        }
        Some(("store", sub_matches)) => match sub_matches.subcommand() {
            Some(("search", sub_matches)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)? {
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let name = sub_matches
                    .value_of("NAME")
                    .expect("When searching the store a package name has to be given.");
                let store = LocalStore::open(STORE_PATH)?;
                for (id, desc, _objects) in store.packages().filter_by_name_containing(name) {
                    println!("{} {desc}\n", style(id.truncate()).blue());
                }
            }
            Some(("collect-garbage", _)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
                {
                    caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let mut store = LocalStore::open(STORE_PATH)?;
                let user_manager = UserManager::open(USER_MANAGER_PATH)?;

                let _removed = store.remove_unused(&user_manager)?;
                store.flush()?;
            }
            // Some(("add", sub_matches)) => {
            //     if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
            //     && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
            //     && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
            // {
            //     caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
            //     caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
            //     caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
            // } else {
            //     return Err("Please run hua init as root or with the appropiate capabilities".into());
            // }

            //     let lock_file = sub_matches
            //         .value_of("LOCK_FILE")
            //         .expect("A lock file has to be provided when adding a package to the store");
            //     let path = sub_matches
            //         .value_of("PATH")
            //         .expect("A path has to be provided when adding a package to the store");

            //     let path = {
            //         let path = PathBuf::from(path);
            //         if path.is_symlink() {
            //             path.read_link()?
            //         } else {
            //             path
            //         }
            //     };

            //     let lock_data = fs::read(lock_file)?;
            //     let package = toml::from_slice::<PackageSource>(&lock_data)?;

            //     info!("Package lock and content read");

            //     let mut store = LocalStore::open(STORE_PATH)?;

            //     info!("Local store opened");

            //     println!("Package to add:\n{}", &package.desc);
            //     if Confirm::new().with_prompt("Continue?").interact()? {
            //         let name = package.name().clone();

            //         store.insert(package, path)?;
            //         store.flush()?;
            //         println!("{} {name} added", style("Success").green());
            //     } else {
            //         println!("Nothing added");
            //     }
            // }
            _ => unreachable!(),
        },
        Some(("generations", sub_matches)) => match sub_matches.subcommand() {
            Some(("list", _)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)? {
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.list_current_generations();
            }
            Some(("current", _)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)? {
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let user_manager = UserManager::open(USER_MANAGER_PATH)?;
                println!("{}", user_manager.current_generation_index());
            }
            Some(("remove", sub_matches)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
                {
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let id = sub_matches
                    .value_of("ID")
                    .expect("When removing a generation, an id has to be given.")
                    .parse()?;
                let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.remove_generation(id)?;
                user_manager.flush()?;

                println!("{} {id} removed", style("Success").green());
            }
            Some(("switch", sub_matches)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
                {
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let id = sub_matches
                    .value_of("ID")
                    .expect("When switching to a generation, an id has to be given.")
                    .parse()?;

                let global_paths = ComponentPathBuf::global();

                let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;
                user_manager.switch_generation(id, &global_paths)?;
                user_manager.flush()?;

                println!("{} switched to {id}", style("Success").green());
            }
            _ => unreachable!(),
        },
        Some(("add", sub_matches)) => {
            if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
                && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
            {
                caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
            } else {
                return Err(
                    "Please run hua init as root or with the appropiate capabilities".into(),
                );
            }

            // TODO should give a selection of packages found in the local store or remote caches and let the user decide

            let path = sub_matches
                .value_of("PATH")
                .expect("When adding a package, a name has to be given.");

            let data = fs::read(path)?;
            let drv = toml::from_slice::<Derivation>(&data)?;

            let mut store = LocalStore::open(STORE_PATH)?;
            let mut user_manager = UserManager::open(USER_MANAGER_PATH)?;
            let config = Config::open(CONFIG_PATH)?;
            let locator = Locator::new(config.to_caches().into_iter())?;

            if store.packages().contains_drv(&drv).is_none() {
                let mut sources = locator.search(&drv).collect::<Vec<_>>();

                if sources.len() > 0 {
                    let selection = Select::new()
                        .with_prompt("Wich package to add (cancel with ESC or q)?")
                        .items(&sources)
                        .interact_opt()?;
                    if let Some(index) = selection {
                        let source = sources.remove(index);
                        store.insert_remote(source)?;
                    } else {
                        println!("Nothing added");
                        return Ok(());
                    }
                } else if PathBuf::from("result").exists() {
                    println!("Please remove result file for building package");
                    return Ok(());
                } else {
                    println!("Building package");
                    let cache = CacheBuilder::default().build()?;
                    let _link = build_recipe(drv.clone(), &mut store, &cache)?;
                }
            }

            info!("Package exists in local store");

            let name = drv.name.clone();
            let id = store.packages().contains_drv(&drv).unwrap();
            let blobs = unsafe { store.get_blobs_cloned_of_package(&id).unwrap_unchecked() };
            let req = (drv, blobs.collect()).into();

            let global_paths = ComponentPathBuf::global();
            user_manager.insert_requirement(req, &store, &global_paths)?;

            info!("Requirement for package added to current user");

            // let global_paths = ComponentPathBuf::global();
            // user_manager.switch_global_links(&global_paths)?;

            //info!("Global links switched to new generation");

            user_manager.flush()?;
            store.flush()?;

            info!("Data persisted to database");

            println!("{} {name} added", style("Success").green());
        }
        Some(("remove", sub_matches)) => {
            if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
            {
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
            } else {
                return Err(
                    "Please run hua init as root or with the appropiate capabilities".into(),
                );
            }

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

                let global_paths = ComponentPathBuf::global();
                user_manager.remove_requirement(&req, &store, &global_paths)?;

                // user_manager.switch_global_links(&global_paths)?;

                user_manager.flush()?;
                println!("{} {name} removed", style("Success").green());
            } else {
                println!("Nothing removed");
            }
        }
        Some(("build", sub_matches)) => {
            if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
                && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
            {
                caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
            } else {
                return Err(
                    "Please run hua init as root or with the appropiate capabilities".into(),
                );
            }

            let path = sub_matches
                .value_of("PATH")
                .expect("A recipe has to be provided.");
            let data = fs::read(path)?;
            let drv = toml::from_slice::<Derivation>(&data)?;

            let mut store = LocalStore::open(STORE_PATH)?;
            let cache = CacheBuilder::default().build()?;

            let link = PathBuf::from("result");
            if link.exists() {
                println!("Please remove the result link.");
                return Ok(());
            }

            if let Some(id) = store.packages().contains_drv(&drv) {
                let path = drv.path_in_store(store.path(), &id);
                unix::fs::symlink(path, &link)?;
                println!("{} {link:#?}", style("Success").green());
            } else {
                let path = recipe::build_recipe(drv, &mut store, &cache)?;

                store.flush()?;
                println!("{} {path:#?}", style("Success").green());
            }
        }
        Some(("shell", sub_matches)) => {
            if caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)? {
                caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
            } else {
                return Err(
                    "Please run hua init as root or with the appropiate capabilities".into(),
                );
            }

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

            let shell_program = match std::env::var("shell") {
                Ok(p) => p,
                Err(_) => "sh".to_owned(),
            };
            let mut child = jail.arg(shell_program).run()?;
            child.wait()?;
        }
        Some(("cache", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
                {
                    caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let url = sub_matches
                    .value_of("URL")
                    .expect("When adding a cache a url has to be provided");
                let url = Url::parse(url)?;

                let mut config = Config::open(CONFIG_PATH)?;
                config.add_cache(url.clone());
                config.flush()?;

                println!("{} {url} added", style("Success").green());
            }
            Some(("remove", _)) => {
                if caps::has_cap(None, CapSet::Permitted, Capability::CAP_CHOWN)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_OVERRIDE)?
                    && caps::has_cap(None, CapSet::Permitted, Capability::CAP_DAC_READ_SEARCH)?
                {
                    caps::raise(None, CapSet::Effective, Capability::CAP_CHOWN)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_OVERRIDE)?;
                    caps::raise(None, CapSet::Effective, Capability::CAP_DAC_READ_SEARCH)?;
                } else {
                    return Err(
                        "Please run hua init as root or with the appropiate capabilities".into(),
                    );
                }

                let mut config = Config::open(CONFIG_PATH)?;

                if config.caches().len() == 0 {
                    println!("Nothing to remove");
                    return Ok(());
                }

                let selection = Select::new()
                    .with_prompt("Wich cache to remove (cancel with ESC or q)?")
                    .items(config.caches())
                    .interact_opt()?;

                if let Some(index) = selection {
                    let removed = config.remove_cache(index);
                    config.flush()?;
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
