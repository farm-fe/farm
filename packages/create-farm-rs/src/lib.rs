use dialoguer::{Confirm, Input, Select};
use std::{ffi::OsString, fs, process::exit};

use crate::{
    package_manager::PackageManager,
    utils::{colors::*, theme::ColorfulTheme},
};

mod args;
mod package_manager;
mod template;
pub mod utils;