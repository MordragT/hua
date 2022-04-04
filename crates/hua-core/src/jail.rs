use std::{
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
};

#[derive(Debug)]
pub enum Bind {
    Dev { src: PathBuf, dest: PathBuf },
    ReadOnly { src: PathBuf, dest: PathBuf },
    ReadWrite { src: PathBuf, dest: PathBuf },
    Symlink { src: PathBuf, dest: PathBuf },
    TmpFs { path: PathBuf },
    Proc { path: PathBuf },
    Dir { path: PathBuf },
    File { content: String, path: PathBuf },
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
    pub fn file<P: AsRef<Path>>(content: &str, path: P) -> Self {
        Self::File {
            content: content.to_owned(),
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
            Self::File { content, path } => cmd.arg("--file").arg(content).arg(path),
        };
    }
}

#[derive(Debug, Default)]
pub struct JailBuilder<'a> {
    args: Vec<&'a str>,
    binds: Vec<Bind>,
    envs: Vec<(&'a str, &'a str)>,
    envs_remove: Vec<(&'a str, &'a str)>,
    env_clear: bool,
    current_dir: Option<PathBuf>,
}

impl<'a> JailBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }
    pub fn args(mut self, args: impl IntoIterator<Item = &'a str>) -> Self {
        self.args.extend(args);
        self
    }
    pub fn bind(mut self, bind: Bind) -> Self {
        self.binds.push(bind);
        self
    }
    pub fn env(mut self, variable: &'a str, value: &'a str) -> Self {
        self.envs.push((variable, value));
        self
    }
    pub fn envs(mut self, vars: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        self.envs.extend(vars);
        self
    }
    pub fn env_remove(mut self, variable: &'a str, value: &'a str) -> Self {
        self.envs_remove.push((variable, value));
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

        if let Some(dir) = self.current_dir {
            bwrap.arg("--chdir").arg(dir);
        }

        for arg in &self.args {
            bwrap.arg(arg);
        }

        bwrap.spawn()
    }
}
