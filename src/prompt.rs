use core::{fmt::Display, num::ParseIntError, ops::Deref, str::FromStr};
use std::{io::ErrorKind, path::PathBuf};

use inquire::{
    autocompletion::Replacement, error::InquireResult, list_option::ListOption, validator::Validation,
    Autocomplete, CustomType, CustomUserError, MultiSelect, Text,
};

use super::error::{Error, Result};

pub fn select_modules<T>(options: Vec<T>) -> InquireResult<Vec<T>>
where
    T: Display,
{
    MultiSelect::new("Select modules:", options)
        .with_validator(|val: &[ListOption<&T>]| {
            if val.is_empty() {
                Ok(Validation::Invalid("modules not be empty".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .with_help_message("It is recommended to choose only one")
        .prompt()
}

pub fn set_depth() -> InquireResult<u8> {
    CustomType::<u8>::new("Set depth:")
        .with_default(7)
        .with_validator(|val: &u8| {
            if *val > 13 {
                Ok(Validation::Invalid(format!("max depth: {}", 13).into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
}

#[derive(Debug, Clone)]
pub struct Offset((u16, u16));

impl Deref for Offset {
    type Target = (u16, u16);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-{}:+{}", self.0 .0, self.0 .1)
    }
}

impl FromStr for Offset {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (lr, ur) = match s.split_once([' ', ':']) {
            Some((lr, ur)) => (lr, ur),
            None => s.split_once('-').ok_or("err")?,
        };
        Ok(Self((lr.trim_start_matches('-').parse()?, ur.trim_start_matches('+').parse()?)))
    }
}

pub fn set_offset() -> InquireResult<Offset> {
    CustomType::<Offset>::new("Set offset:")
        .with_default(Offset((256, 256)))
        .with_validator(|Offset((lr, ur)): &Offset| {
            if lr + ur > 4096 {
                Ok(Validation::Invalid(format!("max offset: {}", 4096).into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
}

#[derive(Debug, Clone)]
pub struct Target(Vec<usize>);

impl Deref for Target {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .fold(Ok(()), |result, n| result.and(write!(f, "{:#x} ", n)))
    }
}

impl FromStr for Target {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split(&[' ', ':', '-'])
                .map(|x| usize::from_str_radix(x.trim_start_matches("0x"), 16))
                .collect::<Result<Vec<_>, ParseIntError>>()?,
        ))
    }
}

pub fn set_target() -> InquireResult<Target> {
    CustomType::<Target>::new("Set target:")
        .with_validator(|val: &Target| {
            if val.is_empty() {
                Ok(Validation::Invalid("target not be empty".into()))
            } else if val.len() > 4 {
                Ok(Validation::Invalid("max number targets is 4".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
}

pub fn set_watch() -> InquireResult<Target> {
    CustomType::<Target>::new("Set watch:")
        .with_validator(|val: &Target| {
            if val.len() > 4 {
                Ok(Validation::Invalid("max number watchs is 4".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
}

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    input: String,
    paths: Vec<String>,
    lcp: String,
}

impl FilePathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        self.input = input.to_owned();
        self.paths.clear();

        let input_path = std::path::PathBuf::from(input);

        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    std::path::PathBuf::from(".")
                } else {
                    p.to_owned()
                }
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let scan_dir = if input.ends_with('/') {
            input_path
        } else {
            fallback_parent.clone()
        };

        let entries = match std::fs::read_dir(scan_dir) {
            Ok(read_dir) => Ok(read_dir),
            Err(err) if err.kind() == ErrorKind::NotFound => std::fs::read_dir(fallback_parent),
            Err(err) => Err(err),
        }?
        .collect::<Result<Vec<_>, _>>()?;

        let mut idx = 0;
        let limit = 15;

        while idx < entries.len() && self.paths.len() < limit {
            let entry = entries.get(idx).unwrap();

            let path = entry.path();
            let path_str = if path.is_dir() {
                format!("{}/", path.to_string_lossy())
            } else {
                path.to_string_lossy().to_string()
            };

            if path_str.starts_with(&self.input) && path_str.len() != self.input.len() {
                self.paths.push(path_str);
            }

            idx = idx.saturating_add(1);
        }

        self.lcp = self.longest_common_prefix();

        Ok(())
    }

    fn longest_common_prefix(&self) -> String {
        let mut ret: String = String::new();

        let mut sorted = self.paths.clone();
        sorted.sort();
        if sorted.is_empty() {
            return ret;
        }

        let mut first_word = sorted.first().unwrap().chars();
        let mut last_word = sorted.last().unwrap().chars();

        loop {
            match (first_word.next(), last_word.next()) {
                (Some(c1), Some(c2)) if c1 == c2 => {
                    ret.push(c1);
                }
                _ => return ret,
            }
        }
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;

        Ok(self.paths.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;

        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => match self.lcp.is_empty() {
                true => Replacement::None,
                false => Replacement::Some(self.lcp.clone()),
            },
        })
    }
}

pub fn set_out_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let input = Text::new("Out dir:")
        .with_autocomplete(FilePathCompleter::default())
        .with_default(&current_dir.to_string_lossy())
        .prompt()?;
    Ok(PathBuf::try_from(input).map_err(|e| e.to_string())?)
}
