use std::fs::{create_dir_all, remove_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use log::{debug, error, info};
use walkdir::WalkDir;

use crate::links::Link;
use crate::options::Options;

#[allow(unused_variables)]
pub fn unlink(options: &Options) -> Vec<Result<(), ()>> {
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);
            if let Ok(links) =
                get_uninstall_paths::<_, &Path>(package, options.target.as_ref(), options.dotfiles)
            {
                for link in links {
                    debug!("{:?}", link);
                    debug!("[TODO] Unwrapping read_link");
                    if link.source.is_dir() {
                        if link.source.read_dir().unwrap().next().is_none() {
                            info!("Directory {:?} is empty, removing...", link.source);
                            let res = remove_dir(link.source);
                            debug!("remove_dir result {:?}", res);
                        }
                    } else if link.source.is_symlink()
                        && link.source.read_link().unwrap() == link.target
                    {
                        info!("Removing link: {:?} -> {:?}", link.source, link.target);
                        let res = remove_file(link.source);
                        debug!("remove_file result {:?}", res);
                    }
                }
            } else {
                error!(
                    "Ran into an error generating the link data for package {:?}",
                    package
                );
                return Err(());
            }
            info!("Done processing package {:?}", package);
            Ok(())
        })
        .collect()
}

pub fn link(options: &Options) -> Vec<Result<(), ()>> {
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);
            if let Ok(links) =
                get_install_paths::<_, &Path>(package, options.target.as_ref(), options.dotfiles)
            {
                for link in links {
                    if link.target.is_dir() {
                        if !link.source.exists() {
                            debug!("Making directory {:?}", link.source);
                            let res = create_dir_all(link.source);
                            debug!("create_dir_all result {:?}", res);
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
                            let res = symlink(link.target, link.source);
                            debug!("symlink result {:?}", res);
                        }
                    }
                }
            } else {
                error!(
                    "Ran into an error generating the link data for package {:?}",
                    package
                );
                return Err(());
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

fn get_install_paths<P, T>(
    package: P,
    target: T,
    map_dots: bool,
) -> Result<Vec<Link>, walkdir::Error>
where
    P: AsRef<Path>,
    T: AsRef<Path>,
{
    WalkDir::new(package)
        .min_depth(1)
        .into_iter()
        .map(|iter| {
            iter.map(|entry| {
                let mut comp = entry.path().components();
                comp.next();
                let source = target.as_ref().join(comp.as_path());
                let mapped_source = match map_dots {
                    true => map_path_dots(source),
                    false => source,
                };

                debug!("[TODO] Unwrapping canonicalize");
                Link {
                    target: entry.path().canonicalize().unwrap().to_path_buf(),
                    source: mapped_source,
                }
            })
        })
        .collect()
}

fn get_uninstall_paths<P, T>(
    package: P,
    target: T,
    map_dots: bool,
) -> Result<Vec<Link>, walkdir::Error>
where
    P: AsRef<Path>,
    T: AsRef<Path>,
{
    WalkDir::new(package)
        .min_depth(1)
        .contents_first(true)
        .into_iter()
        .map(|iter| {
            iter.map(|entry| {
                let mut comp = entry.path().components();
                comp.next();
                let source = target.as_ref().join(comp.as_path());
                let mapped_source = match map_dots {
                    true => map_path_dots(source),
                    false => source,
                };

                debug!("[TODO] Unwrapping canonicalize");
                Link {
                    target: entry.path().canonicalize().unwrap().to_path_buf(),
                    source: mapped_source,
                }
            })
        })
        .collect()
}
