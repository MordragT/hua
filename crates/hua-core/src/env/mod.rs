use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

pub use bind::Bind;
pub use chroot::{Chroot, ChrootChild};

mod bind;
mod chroot;

/// An environemt to execute commands inside
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Environment {
    args: Vec<String>,
    binds: Vec<Bind>,
    envs: Vec<(String, String)>,
    envs_remove: Vec<String>,
    env_clear: bool,
    current_dir: Option<PathBuf>,
}

impl Environment {
    /// Creates a new [Environment].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a [binding](Bind) to the [Environment].
    pub fn bind(&mut self, bind: Bind) -> &mut Self {
        self.binds.push(bind);
        self
    }

    /// Add an environmental variable.
    pub fn env<L: AsRef<str>, R: AsRef<str>>(&mut self, variable: L, value: R) -> &mut Self {
        self.envs
            .push((variable.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }

    /// Add multiple environmental variables.
    pub fn envs<L: AsRef<str>, R: AsRef<str>>(
        &mut self,
        vars: impl IntoIterator<Item = (L, R)>,
    ) -> &mut Self {
        self.envs.extend(
            vars.into_iter()
                .map(|(var, val)| (var.as_ref().to_owned(), val.as_ref().to_owned())),
        );
        self
    }

    /// Remove a environmental of the machine.
    pub fn env_remove<V: AsRef<str>>(&mut self, variable: V) -> &mut Self {
        self.envs_remove.push(variable.as_ref().to_owned());
        self
    }

    /// Clear all environmental variables.
    pub fn env_clear(&mut self, clear: bool) -> &mut Self {
        self.env_clear = clear;
        self
    }

    /// Set the current working directory.
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }

    /// Creates a sandboxed environment for the [Command]
    pub fn build_jail<P: AsRef<OsStr>>(&self, program: P) -> Command {
        let mut bwrap = Command::new("bwrap");

        if self.env_clear {
            bwrap.arg("--clearenv");
        }

        for (var, val) in &self.envs {
            bwrap.arg("--setenv").arg(var).arg(val);
        }

        for var in &self.envs_remove {
            bwrap.arg("--unsetenv").arg(var);
        }

        for bind in &self.binds {
            apply_bind_jail(&bind, &mut bwrap);
        }

        if let Some(dir) = &self.current_dir {
            bwrap.arg("--chdir").arg(dir);
        }

        bwrap.arg(program);

        bwrap
    }

    /// Creates a chrooted environment for a [Command]
    pub fn build_chroot<P: AsRef<OsStr>>(self, program: P) -> Chroot {
        Chroot::new(self, program)
    }
}

fn apply_bind_jail(bind: &Bind, cmd: &mut Command) {
    match bind {
        Bind::Dev { src, dest } => cmd.arg("--dev-bind").arg(src).arg(dest),
        Bind::ReadOnly { src, dest } => cmd.arg("--ro-bind").arg(src).arg(dest),
        Bind::ReadWrite { src, dest } => cmd.arg("--bind").arg(src).arg(dest),
        Bind::TmpFs { path } => cmd.arg("--tmpfs").arg(path),
        Bind::Proc { path } => cmd.arg("--proc").arg(path),
        Bind::Symlink { src, dest } => cmd.arg("--symlink").arg(src).arg(dest),
        Bind::Dir { path } => cmd.arg("--dir").arg(path),
    };
}
