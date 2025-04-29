use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{copy, create_dir_all, remove_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use anyhow::{Result, anyhow};
use path_absolutize::Absolutize;
use walkdir::WalkDir;

use crate::link::Link;
use crate::options::{Command, Options};

/// Performs the actions for a given [Link] when uninstalling a package.
///
/// The actions taken vary depending on if the [Link] target is a directory or a symlink.
fn do_unlink(options: &Options, link: &Link, store: &mut HashMap<PathBuf, PathBuf>) -> Result<()> {
    if link.target.is_dir() {
        if link.target.read_dir()?.next().is_none() {
            info!("Directory {:?} is empty, removing...", link.target);
            if !options.dry_run {
                let res = remove_dir(link.target.as_path());
                store.remove(&link.target);
                debug!("remove_dir result {:?}", res);
            }
        }
    } else if link.target.is_symlink() && link.target.read_link()? == link.source {
        info!("Removing link: {:?} -> {:?}", link.target, link.source);
        if !options.dry_run {
            let res = remove_file(link.target.as_path());
            store.remove(&link.target);
            debug!("remove_file result {:?}", res);
        }
    }
    Ok(())
}

/// Performs the actions for a given [Link] when installing a package.
///
/// The actions taken vary depending on if the [Link] source is a directory or if the target exists
/// or is a symlink.
fn do_link(options: &Options, link: &Link, store: &mut HashMap<PathBuf, PathBuf>) -> Result<()> {
    if link.source.is_dir() {
        debug!("Checking required directory exists {:?}", link.target);
        if link.target.exists() {
            // Mark it as managed
            store.insert(link.target.to_owned(), link.source.to_owned());
            return Ok(());
        }

        // Create directory
        debug!("Creating missing directory {:?}", link.target);
        if !options.dry_run {
            match create_dir_all(link.target.as_path()) {
                Ok(..) => {
                    store.insert(link.target.to_owned(), link.source.to_owned());
                }
                Err(_) => {
                    error!("Failed to create directory {:?}", link.target);
                    // TODO: Should this return an error to abort or should it continue to
                    // link as many files as possible?
                }
            }
        }
        return Ok(());
    }
    // Source not a directory

    info!("Processing link: {:?} -> {:?}", link.target, link.source);
    // Simple case first where no link or file exists at the target
    if !link.target.exists() && !link.target.is_symlink() {
        debug!("Making link {:?} -> {:?}", link.target, link.source);
        if !options.dry_run {
            let res = symlink(link.source.as_path(), link.target.as_path());
            debug!("symlink result {:?}", res);
            // TODO: Match block here
            store.insert(link.target.to_owned(), link.source.to_owned());
        }
        info!("Created link {:?} -> {:?}", link.target, link.source);
        return Ok(());
    }

    // Link exists and points to the right file
    if link.target.is_symlink() && link.target.canonicalize()? == link.source {
        debug!("Link {:?} already exists!", link.target);
        store.insert(link.target.to_owned(), link.source.to_owned());

        // Remake the link if it's a relative link and not absolute
        if link.target.read_link()?.absolutize()? != link.target.read_link()? {
            // TODO: Proper error handling
            let res = remove_file(link.target.as_path());
            debug!("remove result {:?}", res);
            let res = symlink(link.source.as_path(), link.target.as_path());
            debug!("symlink result {:?}", res);
        }
        return Ok(());
    }

    if options.adopt {
        // TODO: Add a confirm/noconfirm option and a y/n prompt
        info!("Found a file at {:?}, adopting...", link.target);

        // Resolve any symlinks, when generating links we don't just generate an absolute
        // path which doesn't follow symlinks
        let target = link.target.canonicalize()?;

        if !options.dry_run {
            // TODO: Better error handling
            match copy(&target, link.source.as_path()) {
                Ok(..) => {
                    // NOTE: Make sure to delete the target and not any potential other files pointed
                    // to by symlink
                    let res = remove_file(link.target.as_path());
                    debug!("remove_file result {:?}", res);
                }
                Err(_e) => {
                    error!("Failed to adopt file {:?}", target)
                }
            }
        }

        debug!("Making link {:?} -> {:?}", link.target, link.source);

        if !options.dry_run {
            // TODO: Better error handling
            let res = symlink(link.source.as_path(), link.target.as_path());
            debug!("symlink result {:?}", res);
            return Ok(());
        }
    }

    // File exists but is not a link to package and we're not adopting so ignore
    Ok(())
}

fn package_error<E>(package: &Path, err: E) -> (PathBuf, anyhow::Error)
where
    E: Into<anyhow::Error>,
{
    (package.to_owned(), err.into())
}

pub fn process_packages(
    options: &Options,
    store: &mut HashMap<PathBuf, PathBuf>,
) -> Vec<core::result::Result<PathBuf, (PathBuf, anyhow::Error)>> {
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);

            // Perform shell expansion on destination name/path
            let target: PathBuf = shellexpand::full(
                options
                    .target
                    .to_str()
                    .ok_or(anyhow!("Could not convert source to str for processing"))
                    .map_err(|err| package_error(package, err))?,
            )
            .map_err(|err| package_error(package, err))?
            .into_owned()
            .into();

            let uninstall = match options.command {
                Command::Link => false,
                Command::Unlink => true,
            };

            let links = get_paths(package, &target, options.dotfiles, uninstall)
                .map_err(|err| package_error(package, err))?;

            let f = match options.command {
                Command::Link => do_link,
                Command::Unlink => do_unlink,
            };

            check_zombies(package, &target, options, store)
                .map_err(|err| package_error(package, err))?;

            for link in links {
                f(options, &link, store).map_err(|err| package_error(package, err))?;
            }

            debug!("Done processing package {:?}", package);
            Ok(package.to_owned())
        })
        .collect()
}

/// Convert "dot-" in a [`Path`] to "."
fn map_path_dots<P>(path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let str = path
        .as_ref()
        .to_str()
        .ok_or(anyhow!("Could not convert path to str"))?;
    let path = str.replace("dot-", ".");
    Ok(PathBuf::from(path))
}

/// Get all of the [`Link`]s for a package. A [`Link`] is generated for each file or directory
/// mapping it to the install location inside the `target` directory on the file system.
///
/// Either all of the [`Link`]s are generated or an `Err` is generated if making one of the
/// [`Link`]s fails.
///
/// `uninstall` causes the [`Link`]s to be generated in the reverse order (files then directories)
/// instead of directories then files.
///
/// `map_dots` calls [map_path_dots] on each of the target files/directories.
fn get_paths(package: &Path, target: &Path, map_dots: bool, uninstall: bool) -> Result<Vec<Link>> {
    let mut links = Vec::new();

    for res in WalkDir::new(package)
        .min_depth(1)
        .contents_first(uninstall)
        .into_iter()
    {
        match res {
            Err(e) => return Err(e.into()),
            Ok(entry) => {
                let comp = entry.path();

                // Remove the current dir from the path
                let path = comp.strip_prefix(package)?;

                // Get absolute path to link origin
                let raw_target = target.join(path);
                let raw_target = match raw_target.absolutize() {
                    Err(e) => {
                        error!(
                            "Could not get absolute path for {:?}. Does the current directory exist?",
                            raw_target
                        );
                        return Err(e.into());
                    }
                    Ok(p) => p.into_owned(),
                };
                let mapped_target = match map_dots {
                    true => map_path_dots(raw_target)?,
                    false => raw_target,
                };

                // Get canonical path to file inside package
                let source = match entry.path().canonicalize() {
                    Err(e) => {
                        error!(
                            "Could not get canonical path of {:?}, does the file/path exist?",
                            entry.path()
                        );
                        return Err(e.into());
                    }
                    Ok(p) => p.to_owned(),
                };

                links.push(Link {
                    source,
                    target: mapped_target.to_path_buf(),
                });
            }
        }
    }

    Ok(links)
}

fn check_zombies(
    package: &Path,
    target: &Path,
    options: &Options,
    store: &mut HashMap<PathBuf, PathBuf>,
) -> Result<()> {
    let mut clean_dirh: HashSet<PathBuf> = HashSet::new();
    let mut clean_dirq: VecDeque<PathBuf> = VecDeque::new();
    // Only needed for dry-run mode
    let mut cleaned_files: HashSet<PathBuf> = HashSet::new();

    info!(
        "Checking destination for dangling links to package {:?}",
        package
    );

    let absolute_target = match target.absolutize() {
        Err(e) => {
            error!(
                "Could not get absolute path of {:?}. Does the current directory exist?",
                target
            );
            return Err(e.into());
        }
        Ok(p) => p,
    };
    let canonicalized_package = match package.canonicalize() {
        Err(e) => {
            error!("Could not get canonicalized path of {:?}.", package);
            return Err(e.into());
        }
        Ok(p) => p,
    };

    let mut keys_to_remove: HashSet<PathBuf> = HashSet::new();
    for entry in store.keys().filter(|i| i.starts_with(&absolute_target)) {
        if store
            .get(entry)
            .unwrap()
            .starts_with(&canonicalized_package)
        {
            debug!(
                "Store entry found for this package+target: {:?}",
                store.get(entry).unwrap()
            );
            match entry.try_exists() {
                Ok(false) => {
                    if !entry.is_symlink() {
                        // file doesn't exist, somehow store is out of sync
                        info!("Couldn't find link for {:?}, removing from store...", entry);
                        keys_to_remove.insert(entry.to_path_buf());
                        continue;
                    }

                    // broken symbolic link
                    let link_dest = match entry.read_link() {
                        Err(e) => {
                            error!("Could not get link destination for {:?}", entry);
                            return Err(e.into());
                        }
                        Ok(p) => p,
                    };

                    if link_dest.starts_with(&canonicalized_package) && !link_dest.exists() {
                        info!("Removing zombie link {:?}", entry);
                        if let Some(parent) = entry.parent() {
                            if !clean_dirh.contains(parent) {
                                clean_dirq.push_back(parent.to_path_buf());
                                clean_dirh.insert(parent.to_path_buf());
                            }
                        }
                        if options.dry_run {
                            cleaned_files.insert(entry.to_path_buf());
                        } else {
                            let res = remove_file(entry);
                            keys_to_remove.insert(entry.to_path_buf());
                            debug!("remove_file result {:?}", res);
                        }
                    }
                }
                Ok(true) => { // dir/target exists so nothing to do
                }
                Err(e) => {
                    error!("Could not check if {:?} exists.", entry);
                    return Err(e.into());
                }
            }
        }
    }
    for key in keys_to_remove {
        store.remove(&key);
    }

    while !clean_dirq.is_empty() {
        let entry = clean_dirq.pop_front().unwrap();

        if store.contains_key(&entry) {
            if !options.dry_run && entry.read_dir().unwrap().next().is_none() {
                info!("Removing zombie dir {:?}", &entry);
                let res = remove_dir(&entry);
                debug!("remove_dir result {:?}", res);
                store.remove(&entry);
                if let Some(parent) = entry.parent() {
                    if !clean_dirh.contains(parent) {
                        clean_dirq.push_back(parent.to_path_buf());
                        clean_dirh.insert(parent.to_path_buf());
                    }
                }
            } else if options.dry_run
                && entry
                    .read_dir()
                    .unwrap()
                    .all(|e| cleaned_files.contains(&e.unwrap().path()))
            {
                info!("Removing zombie dir {:?}", &entry);
                cleaned_files.insert(entry.to_path_buf());
                if let Some(parent) = entry.parent() {
                    if !clean_dirh.contains(parent) {
                        clean_dirq.push_back(parent.to_path_buf());
                        clean_dirh.insert(parent.to_path_buf());
                    }
                }
            }
        }
    }

    Ok(())
}
