use std::collections::HashSet;
use std::fs::{copy, create_dir_all, remove_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use anyhow::{anyhow, bail, Result};
use walkdir::WalkDir;

use crate::link::Link;
use crate::options::{Command, Options};

/// Performs the actions for a given [Link] when uninstalling a package.
///
/// The actions taken vary depending on if the [Link] target is a directory or a symlink.
fn do_unlink(options: &Options, link: &Link) -> Result<()> {
    if link.target.is_dir() {
        if link.target.read_dir()?.next().is_none() {
            info!("Directory {:?} is empty, removing...", link.target);
            if !options.dry_run {
                let res = remove_dir(link.target.as_path());
                debug!("remove_dir result {:?}", res);
            }
        }
    } else if link.target.is_symlink() && link.target.read_link()? == link.source {
        info!("Removing link: {:?} -> {:?}", link.target, link.source);
        if !options.dry_run {
            let res = remove_file(link.target.as_path());
            debug!("remove_file result {:?}", res);
        }
    }
    Ok(())
}

/// Performs the actions for a given [Link] when installing a package.
///
/// The actions taken vary depending on if the [Link] source is a directory or if the target exists
/// or is a symlink.
fn do_link(options: &Options, link: &Link) -> Result<()> {
    if link.source.is_dir() {
        if !link.target.exists() {
            debug!("Making directory {:?}", link.target);
            if !options.dry_run {
                let res = create_dir_all(link.target.as_path());
                debug!("create_dir_all result {:?}", res);
            }
        }
    } else {
        info!("Processing link: {:?} -> {:?}", link.target, link.source);
        if link.target.exists() {
            if link.target.is_symlink() && link.target.read_link()? == link.source {
                debug!("Link {:?} already exists!", link.target);
            } else if options.adopt {
                info!("Found a file at {:?}, adopting...", link.target);
                // TODO: Add a confirm/noconfirm option and a y/n prompt
                let target = link.target.canonicalize()?;

                if !options.dry_run {
                    // If the target is a symlink we should take the target of the symlink unless
                    // the target of the symlink is already the source. Attempting to copy from/to
                    // the same file will cause the file to be truncated!
                    let res = if target != link.source {
                        copy(&target, link.source.as_path())
                    } else {
                        debug!("File was a link pointing to the owned file, skipping copy...");
                        Ok(0)
                    };
                    debug!("copy result {:?}", res);

                    if res.is_ok() {
                        // But make sure to delete the symlink and not the target of the symlink!
                        let res = remove_file(link.target.as_path());
                        debug!("remove_file result {:?}", res);
                    } else {
                        bail!("Failed to adopt file");
                    }
                }

                debug!("Making link {:?} -> {:?}", link.target, link.source);
                if !options.dry_run {
                    let res = symlink(link.source.as_path(), link.target.as_path());
                    debug!("symlink result {:?}", res);
                }
                info!("Created link {:?} -> {:?}", link.target, link.source);
            } else {
                error!("Item already exists at link location! {:?}", link.target);
            }
        } else if !link.target.exists() {
            debug!("Making link {:?} -> {:?}", link.target, link.source);
            if !options.dry_run {
                let res = symlink(link.source.as_path(), link.target.as_path());
                debug!("symlink result {:?}", res);
            }
            info!("Created link {:?} -> {:?}", link.target, link.source);
        }
    }
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

            check_zombies(package, &target, options).map_err(|err| package_error(package, err))?;

            for link in links {
                f(options, &link).map_err(|err| package_error(package, err))?;
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

                // Get path to link origin
                let raw_target = target.join(path);
                let mapped_target = match map_dots {
                    true => map_path_dots(raw_target)?,
                    false => raw_target,
                };

                // Get absolute path to file inside package
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

fn check_zombies(package: &Path, target: &Path, options: &Options) -> Result<()> {
    let mut clean_dirs: HashSet<PathBuf> = HashSet::new();
    // Only needed for dry-run mode
    let mut cleaned_files: HashSet<PathBuf> = HashSet::new();

    info!("Checking destination for dangling links to package {:?}", package);

    for res in WalkDir::new(target)
        .min_depth(1)
        .contents_first(true)
        .into_iter()
    {
        match res {
            Err(e) => error!("Encountered error: {:?}", e),
            Ok(entry) => {
                if entry.path_is_symlink() {
                    let link_dest = entry.path().read_link()?;

                    if link_dest.starts_with(package.canonicalize()?) && !link_dest.exists() {
                        info!("Removing zombie link {:?}", entry.path());
                        if let Some(parent) = entry.path().parent() {
                            clean_dirs.insert(parent.canonicalize()?.to_path_buf());
                        }
                        if options.dry_run {
                            cleaned_files.insert(entry.path().to_path_buf());
                        } else {
                            let res = remove_file(entry.path());
                            debug!("remove_file result {:?}", res);
                        }
                    }
                } else if entry.path().is_dir() {
                    debug!("Checking directory: {:?}", entry.path());
                    if clean_dirs.contains(entry.path().canonicalize()?.as_path()) {
                        if options.dry_run {
                            if entry.path().read_dir()?.fold(true, |state, elem| {
                                cleaned_files.contains(&elem.unwrap().path()) & state
                            }) {
                                info!("Removing zombie dir {:?}", entry.path());
                            }
                        } else if entry.path().read_dir()?.next().is_none() {
                            info!("Removing zombie dir {:?}", entry.path());
                            let res = remove_dir(entry.path());
                            debug!("remove_dir result {:?}", res);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
