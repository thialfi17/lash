use std::fs::{create_dir_all, remove_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::{debug, error, info};
use walkdir::WalkDir;

use crate::links::Link;
use crate::options::Options;

#[allow(unused_variables)]
pub fn unlink(options: &Options) -> Vec<Result<()>> {
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);
            if let Ok(links) =
                get_paths::<_, &Path>(package, options.target.as_ref(), options.dotfiles, true)
            {
                for link in links {
                    debug!("{:?}", link);
                    debug!("[TODO] Unwrapping read_link");
                    if link.source.is_dir() {
                        if link.source.read_dir().unwrap().next().is_none() {
                            info!("Directory {:?} is empty, removing...", link.source);
                            if !options.dry_run {
                                let res = remove_dir(link.source);
                                debug!("remove_dir result {:?}", res);
                            }
                        }
                    } else if link.source.is_symlink()
                        && link.source.read_link().unwrap() == link.target
                    {
                        info!("Removing link: {:?} -> {:?}", link.source, link.target);
                        if !options.dry_run {
                            let res = remove_file(link.source);
                            debug!("remove_file result {:?}", res);
                        }
                    }
                }
            } else {
                anyhow::bail!(
                    "Ran into an error generating the link data for package {:?}",
                    package
                );
            }
            info!("Done processing package {:?}", package);
            Ok(())
        })
        .collect()
}

pub fn link(options: &Options) -> Vec<Result<()>> {
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);
            if let Ok(links) =
                get_paths::<_, &Path>(package, options.target.as_ref(), options.dotfiles, false)
            {
                for link in links {
                    if link.target.is_dir() {
                        if !link.source.exists() {
                            debug!("Making directory {:?}", link.source);
                            if !options.dry_run {
                                let res = create_dir_all(link.source);
                                debug!("create_dir_all result {:?}", res);
                            }
                        }
                    } else {
                        info!("Processing link: {:?} -> {:?}", link.source, link.target);
                        if link.source.exists() {
                            debug!("[TODO] Unwrapping read_link");
                            if link.source.is_symlink()
                                && link.source.read_link().unwrap() == link.target
                            {
                                info!("Link {:?} already exists!", link.source)
                            } else if options.adopt {
                                todo!("Implement adopt");
                            } else {
                                error!("Item already exists at link location! {:?}", link.source);
                            }
                        } else if !link.source.exists() {
                            debug!("Making link {:?} -> {:?}", link.source, link.target);
                            if !options.dry_run {
                                let res = symlink(link.target, link.source);
                                debug!("symlink result {:?}", res);
                            }
                        }
                    }
                }
            } else {
                anyhow::bail!(
                    "Ran into an error generating the link data for package {:?}",
                    package
                );
            }
            info!("Done processing package {:?}", package);
            Ok(())
        })
        .collect()
}

/// Convert "dot-" in [`Path`]s to "."
fn map_path_dots<P>(path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let path = match path.as_ref().to_str() {
        Some(str) => str.replace("dot-", "."),
        None => panic!("Couldn't convert path to str"),
    };
    PathBuf::from(path)
}

fn get_paths<P, T>(
    package: P,
    target: T,
    map_dots: bool,
    uninstall: bool,
) -> Result<Vec<Link>, walkdir::Error>
where
    P: AsRef<Path>,
    T: AsRef<Path>,
{
    WalkDir::new(package)
        .min_depth(1)
        .contents_first(uninstall)
        .into_iter()
        .map(|iter| {
            iter.map(|entry| {
                let mut comp = entry.path().components();
                comp.next();
                let raw_source = target.as_ref().join(comp.as_path());
                let mapped_source = match map_dots {
                    true => map_path_dots(raw_source),
                    false => raw_source,
                };
                let source = shellexpand::full(mapped_source.to_str().expect("")).unwrap();

                debug!("[TODO] Unwrapping canonicalize");
                Link {
                    target: entry.path().canonicalize().unwrap().to_path_buf(),
                    source: source.into_owned().into(),
                }
            })
        })
        .collect()
}
