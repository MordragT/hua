use std::{
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
};

use super::{Bind, Environment};

/// A chrooted environment for a [Command](std::process::Command).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chroot {
    environment: Environment,
    program: OsString,
    args: Vec<String>,
}

impl Chroot {
    /// Creates a new chrooted environment.
    pub fn new<P: AsRef<OsStr>>(environment: Environment, program: P) -> Self {
        Self {
            environment,
            program: program.as_ref().to_owned(),
            args: Vec::new(),
        }
    }

    /// Add an argument to the chrooted environment.
    pub fn arg<A: AsRef<str>>(&mut self, arg: A) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Add multiple arguments to the chrooted environment.
    pub fn args<'a>(&mut self, args: impl IntoIterator<Item = &'a str>) -> &mut Self {
        self.args.extend(args.into_iter().map(str::to_string));
        self
    }

    /// Creates all the directories and files for the chrooted environment
    /// under the given path and spawn a [ChrootChild] inside it.
    pub fn spawn<P: AsRef<Path>>(&self, under: P) -> io::Result<ChrootChild> {
        let path = under.as_ref();

        for bind in &self.environment.binds {
            self.apply_bind(&bind, path)?;
        }

        let mut command = Command::new(&self.program);

        if self.environment.env_clear {
            command.env_clear();
        }

        command.envs(self.environment.envs.iter().map(|(var, val)| (var, val)));

        for var in &self.environment.envs_remove {
            command.env_remove(var);
        }

        if let Some(dir) = &self.environment.current_dir {
            command.current_dir(dir);
        }

        let child = command.spawn()?;

        Ok(ChrootChild::new(path.to_owned(), child))
    }

    fn apply_bind(&self, _bind: &Bind, _path: &Path) -> io::Result<()> {
        todo!()
        // match bind {
        //     Bind::Dev { src, dest } => cmd.arg("--dev-bind").arg(src).arg(dest),
        //     Bind::ReadOnly { src, dest } => cmd.arg("--ro-bind").arg(src).arg(dest),
        //     Bind::ReadWrite { src, dest } => cmd.arg("--bind").arg(src).arg(dest),
        //     Bind::TmpFs { path } => cmd.arg("--tmpfs").arg(path),
        //     Bind::Proc { path } => cmd.arg("--proc").arg(path),
        //     Bind::Symlink { src, dest } => cmd.arg("--symlink").arg(src).arg(dest),
        //     Bind::Dir { path } => cmd.arg("--dir").arg(path),
        // }
    }
}

/// Representation of a running or exited process
/// in a chrooted environment.
pub struct ChrootChild {
    path: PathBuf,
    child: Child,
}

impl ChrootChild {
    fn new(path: PathBuf, child: Child) -> Self {
        Self { path, child }
    }

    /// Waits for the child process to exit completly.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }

    /// Forces the child process to exit immediatly.
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }
}

impl Drop for ChrootChild {
    fn drop(&mut self) {
        todo!()
    }
}
