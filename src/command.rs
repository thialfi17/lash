use std::fs::{create_dir_all, remove_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::{debug, error, info};
use walkdir::WalkDir;

use crate::link::Link;
use crate::options::Options;

pub fn unlink(options: &Options) -> Vec<Result<()>> {
    map_packages(options, do_unlink)
}

pub fn link(options: &Options) -> Vec<Result<()>> {
    map_packages(options, do_link)
}

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
                info!("Link {:?} already exists!", link.target)
            } else if options.adopt {
                todo!("Implement adopt");
            } else {
                error!("Item already exists at link location! {:?}", link.target);
            }
        } else if !link.target.exists() {
            debug!("Making link {:?} -> {:?}", link.target, link.source);
            if !options.dry_run {
                let res = symlink(link.source.as_path(), link.target.as_path());
                debug!("symlink result {:?}", res);
            }
        }
    }
    Ok(())
}

/// Run function `f` on each of the packages provided. This function performs the generic
/// processing for each package that is independent of if the package is being
/// installed/uninstalled.
fn map_packages<F>(options: &Options, f: F) -> Vec<Result<()>>
where
    F: Fn(&Options, &Link) -> Result<()>,
{
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);

            // Perform shell expansion on destination name/path
            let target: PathBuf = shellexpand::full(options.target.to_str().ok_or(
                anyhow::anyhow!("Could not convert source to str for processing"),
            )?)?
            .into_owned()
            .into();

            let links = get_paths(package, &target, options.dotfiles, false)?;

            for link in links {
                f(options, &link)?;
            }

            info!("Done processing package {:?}", package);
            Ok(())
        })
        .collect()
}

/// Convert "dot-" in a [`Path`] to "."
fn map_path_dots<P>(path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    log::warn!("TODO not handling failing to_str() conversion correctly!");
    let path = match path.as_ref().to_str() {
        Some(str) => str.replace("dot-", "."),
        None => panic!("Couldn't convert path to str"),
    };
    PathBuf::from(path)
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
fn get_paths(package: &PathBuf, target: &PathBuf, map_dots: bool, uninstall: bool) -> Result<Vec<Link>>
{
    let mut links = Vec::new();

    for res in WalkDir::new(package.as_path())
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
                log::debug!("Stripped path is {:?}", path);

                // Get path to link origin
                let raw_target = target.as_path().join(path);
                let mapped_target = match map_dots {
                    true => map_path_dots(raw_target),
                    false => raw_target,
                };

                // Get absolute path to file inside package
                let source = entry.path().canonicalize()?.to_path_buf();

                links.push(Link {
                    source,
                    target: mapped_target.to_path_buf(),
                });
            }
        }
    }

    Ok(links)
}
