use crate::glue;
use itertools::Itertools;
use roc_std::{RocList, RocStr};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Id(u64);

impl From<&glue::Job> for Id {
    /// We don't care about order in some places (e.g. output file) while we do
    /// in others (e.g. command arguments.) The hash should reflect this!
    ///
    /// Note: this data structure is going to grow the ability to refer to other
    /// jobs as soon as it's feasible. When that happens, a depth-first search
    /// through the tree rooted at `top_job` will probably suffice.
    fn from(top_job: &glue::Job) -> Self {
        // TODO: is this the best hash for this kind of data? Should we find
        // a faster one?
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        let job = &top_job.f0;

        // TODO: when we can get commands from other jobs, we need to hash the
        // other tool and job instead of relying on the derived `Hash` trait
        // for this.
        job.command.hash(&mut hasher);

        // TODO: input file hashes need to change this hash. We cannot do that
        // yet, so we cannot accept files yet!
        debug_assert!(
            job.inputFiles.is_empty(),
            "we cannot handle input files in hashes yet"
        );

        job.outputs
            .iter()
            .sorted()
            .for_each(|output| output.hash(&mut hasher));

        Id(hasher.finish())
    }
}

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
    pub outputs: RocList<RocStr>,
}

impl From<glue::Job> for Job {
    fn from(job: glue::Job) -> Self {
        let id = Id::from(&job);
        let unwrapped = job.f0;

        Job {
            id,
            command: unwrapped.command.f0,
            input_files: unwrapped
                .inputFiles
                .iter()
                .map(|s| s.as_str().into())
                .collect(),
            outputs: unwrapped.outputs,
        }
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
