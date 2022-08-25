use crate::glue;
use anyhow::Result;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::path::{Component, PathBuf};
use std::process::Command;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Id(u64);

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug)]
pub struct Job {
    pub id: Id,
    pub command: glue::R3,
    pub input_files: HashSet<PathBuf>,
    pub outputs: HashSet<PathBuf>,
}

impl Job {
    pub fn from_glue(job: glue::Job, path_to_hash: &HashMap<PathBuf, String>) -> Result<Self> {
        let unwrapped = job.f0;

        // TODO: is this the best hash for this kind of data? Should we find
        // a faster one?
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        // TODO: when we can get commands from other jobs, we need to hash the
        // other tool and job instead of relying on the derived `Hash` trait
        // for this.
        unwrapped.command.hash(&mut hasher);

        let mut input_files: HashSet<PathBuf> = HashSet::with_capacity(unwrapped.inputFiles.len());
        for path_str in unwrapped.inputFiles.iter().sorted() {
            let path = PathBuf::from(path_str.as_str());

            match path_to_hash.get(&path) {
                Some(hash) => hash.hash(&mut hasher),
                None => anyhow::bail!("couldn't find a hash for `{}`", path.display()),
            }

            input_files.insert(path);
        }

        let mut outputs = HashSet::new();
        for output_str in unwrapped.outputs.iter().sorted() {
            let output = PathBuf::from(output_str.as_str());

            if outputs.contains(&output) {
                continue;
            }

            // verify that the specified path is safe. We can't allow accessing
            // any path outside the workspace. To get this, we don't allow
            // any parent path segments (`..`) This restriction also enforces
            // unambiguous paths in the Roc API (e.g. you wouldn't want to add
            // "foo/../bar" as an output path!)
            for component in output.components() {
                match component {
                    Component::Prefix(_) | Component::RootDir => anyhow::bail!(
                        "Got `{}` as an output, but absolute paths are not allowed as outputs. Remove the absolute prefix to fix this!",
                        output.display(),
                    ),

                    Component::ParentDir => anyhow::bail!(
                        "Got `{}` as an output, but relative paths containing `..` are not allowed as inputs. Remove the `..` to fix this!",
                        output.display(),
                    ),

                    Component::CurDir | Component::Normal(_) => (),
                };
            }

            output.hash(&mut hasher);
            outputs.insert(output);
        }

        Ok(Job {
            id: Id(hasher.finish()),
            command: unwrapped.command.f0,
            input_files,
            outputs,
        })
    }
}

impl From<&Job> for Command {
    fn from(job: &Job) -> Self {
        let mut command = Command::new(&job.command.tool.f0.to_string());

        for arg in &job.command.args {
            command.arg(arg.as_str());
        }

        command
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // intention: make a best-effort version of part of how the command
        // would look if it were invoked from a shell. It's OK for right now
        // if it wouldn't work (due to unescaped quotes or whatever.) Point is
        // for us to have some human-readable output in addition to the ID.
        let mut chars = 0;

        write!(f, "{} (", self.id)?;

        let base = self.command.tool.f0.to_string();
        chars += base.len();

        write!(f, "{}", base)?;

        for arg in &self.command.args {
            if chars >= 20 {
                continue;
            }

            if arg.contains(' ') {
                write!(f, " \"{}\"", arg)?;
                chars += arg.len() + 3;
            } else {
                write!(f, " {}", arg)?;
                chars += arg.len() + 1;
            }
        }

        write!(f, ")")
    }
}
