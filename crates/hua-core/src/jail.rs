use std::{
    io,
    os::unix::prelude::RawFd,
    path::{Path, PathBuf},
    process::{Child, Command},
};

use command_fds::{CommandFdExt, FdMapping};

#[derive(Debug)]
pub enum Bind {
    Dev { src: PathBuf, dest: PathBuf },
    ReadOnly { src: PathBuf, dest: PathBuf },
    ReadWrite { src: PathBuf, dest: PathBuf },
    Symlink { src: PathBuf, dest: PathBuf },
    TmpFs { path: PathBuf },
    Proc { path: PathBuf },
    Dir { path: PathBuf },
}

impl Bind {
    pub fn dev<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::Dev {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }
    pub fn read_only<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::ReadOnly {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }
    pub fn read_write<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::ReadWrite {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }
    pub fn symlink<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::Symlink {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }
    pub fn tmp_fs<P: AsRef<Path>>(path: P) -> Self {
        Self::TmpFs {
            path: path.as_ref().to_owned(),
        }
    }
    pub fn proc<P: AsRef<Path>>(path: P) -> Self {
        Self::Proc {
            path: path.as_ref().to_owned(),
        }
    }
    pub fn dir<P: AsRef<Path>>(path: P) -> Self {
        Self::Dir {
            path: path.as_ref().to_owned(),
        }
    }
    pub fn apply(&self, cmd: &mut Command) {
        match self {
            Self::Dev { src, dest } => cmd.arg("--dev-bind").arg(src).arg(dest),
            Self::ReadOnly { src, dest } => cmd.arg("--ro-bind").arg(src).arg(dest),
            Self::ReadWrite { src, dest } => cmd.arg("--bind").arg(src).arg(dest),
            Self::Symlink { src, dest } => cmd.arg("--symlink").arg(src).arg(dest),
            Self::TmpFs { path } => cmd.arg("--tmpfs").arg(path),
            Self::Proc { path } => cmd.arg("--proc").arg(path),
            Self::Dir { path } => cmd.arg("--dir").arg(path),
        };
    }
}

#[derive(Debug)]
struct File {
    child_fd: RawFd,
    path: PathBuf,
    permission: u32,
}

#[derive(Debug)]
pub struct JailBuilder {
    args: Vec<String>,
    binds: Vec<Bind>,
    envs: Vec<(String, String)>,
    envs_remove: Vec<(String, String)>,
    env_clear: bool,
    current_dir: Option<PathBuf>,
    fd_mappings: Vec<FdMapping>,
    files: Vec<File>,
    fd_counter: RawFd,
}

impl Default for JailBuilder {
    fn default() -> Self {
        Self {
            args: Vec::new(),
            binds: Vec::new(),
            envs: Vec::new(),
            envs_remove: Vec::new(),
            env_clear: false,
            current_dir: None,
            fd_mappings: Vec::new(),
            files: Vec::new(),
            fd_counter: 150,
        }
    }
}

impl JailBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn arg<A: AsRef<str>>(mut self, arg: A) -> Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }
    pub fn args<A: AsRef<str>>(mut self, args: impl IntoIterator<Item = A>) -> Self {
        self.args
            .extend(args.into_iter().map(|a| a.as_ref().to_owned()));
        self
    }
    pub fn bind(mut self, bind: Bind) -> Self {
        self.binds.push(bind);
        self
    }
    // TODO not working fix or remove
    pub fn file<P: AsRef<Path>>(mut self, parent_fd: RawFd, path: P, permission: u32) -> Self {
        self.fd_counter += 1;
        let child_fd = self.fd_counter;
        let file = File {
            child_fd,
            path: path.as_ref().to_owned(),
            permission,
        };
        let mapping = FdMapping {
            parent_fd,
            child_fd,
        };

        self.fd_mappings.push(mapping);
        self.files.push(file);

        self
    }
    pub fn env<L: AsRef<str>, R: AsRef<str>>(mut self, variable: L, value: R) -> Self {
        self.envs
            .push((variable.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }
    pub fn envs<L: AsRef<str>, R: AsRef<str>>(
        mut self,
        vars: impl IntoIterator<Item = (L, R)>,
    ) -> Self {
        self.envs.extend(
            vars.into_iter()
                .map(|(var, val)| (var.as_ref().to_owned(), val.as_ref().to_owned())),
        );
        self
    }
    pub fn env_remove<L: AsRef<str>, R: AsRef<str>>(mut self, variable: L, value: R) -> Self {
        self.envs_remove
            .push((variable.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }
    pub fn env_clear(mut self, clear: bool) -> Self {
        self.env_clear = clear;
        self
    }
    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }
    pub fn run(self) -> io::Result<Child> {
        let mut bwrap = Command::new("bwrap");

        if self.env_clear {
            bwrap.arg("--clearenv");
        }

        for (var, val) in self.envs {
            bwrap.arg("--setenv").arg(var).arg(val);
        }

        for (var, val) in self.envs_remove {
            bwrap.arg("--unsetenv").arg(var).arg(val);
        }

        for bind in self.binds {
            bind.apply(&mut bwrap);
        }

        for file in self.files {
            bwrap
                .arg("--perms")
                .arg(file.permission.to_string())
                .arg("--file")
                .arg(file.child_fd.to_string())
                .arg(file.path);
        }

        if let Some(dir) = self.current_dir {
            bwrap.arg("--chdir").arg(dir);
        }

        for arg in &self.args {
            bwrap.arg(arg);
        }

        bwrap
            .fd_mappings(self.fd_mappings)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        bwrap.spawn()
    }
}
