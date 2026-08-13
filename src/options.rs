use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
    sync::Arc,
};

use nu_protocol::{LabeledError, Span};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Engine {
    Dua,
    Jwalk,
    Zlob,
}

impl Engine {
    pub fn parse(value: Option<&str>, span: Span) -> Result<Self, LabeledError> {
        match value {
            None | Some("dua") => Ok(Self::Dua),
            Some("jwalk") => Ok(Self::Jwalk),
            Some("zlob") => Ok(Self::Zlob),
            Some(other) => Err(LabeledError::new("invalid engine").with_label(
                format!("expected 'dua', 'jwalk', or 'zlob', got '{other}'"),
                span,
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dua => "dua",
            Self::Jwalk => "jwalk",
            Self::Zlob => "zlob",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalkOrder {
    Completion,
    ParentFirst,
}

impl WalkOrder {
    pub fn parse(value: Option<&str>, span: Span) -> Result<Self, LabeledError> {
        match value {
            None | Some("completion") => Ok(Self::Completion),
            Some("parent-first") => Ok(Self::ParentFirst),
            Some(other) => Err(LabeledError::new("invalid order").with_label(
                format!("expected 'completion' or 'parent-first', got '{other}'"),
                span,
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completion => "completion",
            Self::ParentFirst => "parent-first",
        }
    }
}

#[derive(Clone, Debug)]
pub struct WalkOptions {
    pub engine: Engine,
    pub path: PathBuf,
    pub span: Span,
    pub sort: bool,
    pub custom: bool,
    pub skip_hidden: bool,
    pub follow_links: bool,
    pub min_depth: usize,
    pub max_depth: usize,
    pub threads: Option<usize>,
    pub skip_dirs: Arc<[OsString]>,
    pub verbose: bool,
    pub metadata: bool,
    pub count: bool,
    pub debug: bool,
    pub order: WalkOrder,
}

impl WalkOptions {
    pub fn records(&self) -> bool {
        self.verbose || self.metadata
    }

    pub fn validate(&self) -> Result<(), LabeledError> {
        if self.engine == Engine::Dua && self.follow_links {
            return Err(
                LabeledError::new("follow-links requires the jwalk engine").with_label(
                    "dua never follows symbolic links; pass --engine jwalk",
                    self.span,
                ),
            );
        }
        if self.engine != Engine::Jwalk && self.custom {
            return Err(
                LabeledError::new("custom walker requires the jwalk engine").with_label(
                    "--custom uses jwalk process_read_dir; pass --engine jwalk --verbose",
                    self.span,
                ),
            );
        }
        if self.custom && !self.records() {
            return Err(LabeledError::new("Please remove the custom flag")
                .with_label("Custom walker only supported with verbose mode", self.span));
        }
        Ok(())
    }

    pub fn should_skip_dir_name(&self, name: &OsStr) -> bool {
        self.skip_dirs.iter().any(|skip| skip.as_os_str() == name)
    }

    pub fn debug_summary(&self, elapsed: std::time::Duration, count: Option<u64>) -> String {
        let count_line = match count {
            Some(n) => format!("\n  count: {n}"),
            None => String::new(),
        };
        format!(
            "Running with these options:\n  engine: {}\n  order: {}\n  sort: {}\n  skip_hidden: {}\n  follow_links: {}\n  min_depth: {}\n  max_depth: {}\n  threads: {:?}\n  skip_dirs: {:?}\n  metadata: {}\nTime: {elapsed:?}{count_line}",
            self.engine.as_str(),
            self.order.as_str(),
            self.sort,
            self.skip_hidden,
            self.follow_links,
            self.min_depth,
            self.max_depth,
            self.threads,
            self.skip_dirs,
            self.metadata,
        )
    }
}

pub fn is_hidden_name(name: &OsStr) -> bool {
    name.as_encoded_bytes().first().copied() == Some(b'.')
}
