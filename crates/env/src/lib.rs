//! Useful environment variables in Alfred workflows.
//!
//! See <https://www.alfredapp.com/help/workflows/script-environment-variables/>
//!
//! # Usage
//!
//! This crate is re-exported from the `powerpack` crate, so you can access it
//! using `powerpack::env`.
//!
//! ```no_run
//! # mod powerpack { pub extern crate powerpack_env as env; } // mock re-export
//! use powerpack::env;
//!
//! let cache_dir = env::workflow_cache();
//! ```

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// Fetches the environment variable `key` from the current process.
///
/// This function is similar to [`std::env::var(key).ok()`][std::env::var] but
/// it also maps an empty string to `None`.
///
/// # None
///
/// Returns `None` in the following cases:
/// - if the environment variable is not present.
/// - if the environment variable is not valid Unicode.
/// - if the environment variable is set to an empty string.
#[inline]
pub fn var<K: AsRef<OsStr>>(key: K) -> Option<String> {
    env::var(key).ok().filter(|s| !s.is_empty())
}

/// Fetches the environment variable `key` from the current process.
///
/// This function is similar to [`std::env::var_os(key).ok()`][std::env::var]
/// but it also maps an empty string to `None`.
///
/// # None
///
/// Returns `None` in the following cases:
/// - if the environment variable is not present.
/// - if the environment variable is set to an empty string.
///
/// Note that the method will not check if the environment variable is valid
/// Unicode. If you want to return `None` on invalid UTF-8, use the [`var`]
/// function instead.
#[inline]
pub fn var_os<K: AsRef<OsStr>>(key: K) -> Option<OsString> {
    env::var_os(key).filter(|s| !s.is_empty())
}

/// Whether or not the user currently has the Alfred debug panel open.
#[inline]
pub fn is_debug() -> bool {
    var("alfred_debug").as_deref() == Some("1")
}

/// The location of the `Alfred.alfredpreferences` directory.
///
/// If a user has synced their settings, this will allow you to find out where
/// their settings are.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let prefs = env::preferences().unwrap();
/// println!("Alfred Preferences:\n{prefs:?}");
/// // Alfred Preferences:
/// // /Users/John/Library/Application Support/Alfred/Alfred.alfredpreferences
///
pub fn preferences() -> Option<PathBuf> {
    var_os("alfred_preferences").map(PathBuf::from)
}

/// The Alfred version that is currently running.
///
/// This may be useful if your workflow depends on particular Alfred features.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let version = env::version().unwrap();
/// println!("Alfred Version: {version}");
/// // Alfred Version: 5.5.1
#[inline]
pub fn version() -> Option<String> {
    var("alfred_version")
}

/// The Alfred build version that is currently running.
///
/// This may be useful if your workflow depends on particular Alfred features.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let build = env::version_build().unwrap();
/// println!("Alfred Build: {build}");
/// // Alfred Build: 2273
/// ```
#[inline]
pub fn version_build() -> Option<u32> {
    var("alfred_version_build").and_then(|s| s.parse().ok())
}

/// The bundle ID of the currently running workflow.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let bundle_id = env::workflow_bundle_id().unwrap();
/// println!("Workflow Bundle ID: {bundle_id}");
/// // Workflow Bundle ID: com.example.workflow
#[inline]
pub fn workflow_bundle_id() -> Option<String> {
    var("alfred_workflow_bundleid")
}

/// The name of the currently running workflow.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let name = env::workflow_name().unwrap();
/// println!("Workflow Name: {name}");
/// // Workflow Name: Example Workflow
#[inline]
pub fn workflow_name() -> Option<String> {
    var("alfred_workflow_name")
}

/// The unique ID of the currently running workflow.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let uid = env::workflow_uid().unwrap();
/// println!("Workflow UID: {uid}");
/// // Workflow UID: user.workflow.B0AC54EC-601C-479A-9428-01F9FD732959
/// ```
#[inline]
pub fn workflow_uid() -> Option<String> {
    var("alfred_workflow_uid")
}

/// The version of the currently running workflow.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let version = env::workflow_version().unwrap();
/// println!("Workflow Version: {version}");
/// // Workflow Version: 1.2.3
#[inline]
pub fn workflow_version() -> Option<String> {
    var("alfred_workflow_version")
}

/// The recommended directory for volatile workflow data.
///
/// This will only be populated if your workflow has a bundle id set.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let cache = env::workflow_cache().unwrap();
/// println!("Workflow Cache:\n{cache:?}");
/// // Workflow Cache:
/// // /Users/John/Library/Caches/com.runningwithcrayons.Alfred/Workflow Data/com.example.workflow
#[inline]
pub fn workflow_cache() -> Option<PathBuf> {
    var_os("alfred_workflow_cache").map(PathBuf::from)
}

/// The workflow cache directory or sensible default value.
///
/// # Panics
///
/// Panics if the user's home directory cannot be determined.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let cache = env::workflow_cache_or_default();
/// println!("Workflow Cache:\n{cache:?}");
/// // Workflow Cache:
/// // /Users/John/Library/Caches/com.runningwithcrayons.Alfred/Workflow Data/com.example.workflow
#[inline]
pub fn workflow_cache_or_default() -> PathBuf {
    try_workflow_cache_or_default().expect("no home directory found")
}

/// The workflow cache directory or sensible default value.
///
/// # None
///
/// Returns `None` if the user's home directory cannot be determined.
#[inline]
pub fn try_workflow_cache_or_default() -> Option<PathBuf> {
    workflow_cache().or_else(|| {
        let mut d = workflow_cache_home()?;
        let bundle_id = workflow_bundle_id();
        d.push(bundle_id.as_deref().unwrap_or("powerpack"));
        Some(d)
    })
}
fn workflow_cache_home() -> Option<PathBuf> {
    let mut d = home::home_dir()?;
    d.extend([
        "Library",
        "Caches",
        "com.runningwithcrayons.Alfred",
        "Workflow Data",
    ]);
    Some(d)
}

/// The recommended directory for non-volatile workflow data.
///
/// This will only be populated if your workflow has a bundle id set.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let data = env::workflow_data().unwrap();
/// println!("Workflow Data:\n{data:?}");
/// // Workflow Data:
/// // /Users/John/Library/Application Support/Alfred/Workflow Data/com.example.workflow
#[inline]
pub fn workflow_data() -> Option<PathBuf> {
    var_os("alfred_workflow_data").map(PathBuf::from)
}

/// The workflow data directory or sensible default value.
///
/// # Panics
///
/// Panics if the user's home directory cannot be determined.
///
/// # Examples
///
/// ```no_run
/// # use powerpack_env as env;
/// let data = env::workflow_data_or_default();
/// println!("Workflow Data:\n{data:?}");
/// // Workflow Data:
/// // /Users/John/Library/Application Support/Alfred/Workflow Data/com.example.workflow
#[inline]
pub fn workflow_data_or_default() -> PathBuf {
    try_workflow_data_or_default().expect("no home directory found")
}
/// The workflow data directory or sensible default value.
///
/// # None
///
/// Returns `None` if the user's home directory cannot be determined.
#[inline]
pub fn try_workflow_data_or_default() -> Option<PathBuf> {
    workflow_data().or_else(|| {
        let mut d = workflow_data_home()?;
        let bundle_id = workflow_bundle_id();
        d.push(bundle_id.as_deref().unwrap_or("powerpack"));
        Some(d)
    })
}
fn workflow_data_home() -> Option<PathBuf> {
    let mut d = home::home_dir()?;
    d.extend(["Library", "Application Support", "Alfred", "Workflow Data"]);
    Some(d)
}
