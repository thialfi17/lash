use std::fs::{create_dir_all, remove_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::{debug, error, info};
use walkdir::WalkDir;

use crate::links::Link;
use crate::options::Options;

pub fn unlink(options: &Options) -> Vec<Result<()>> {
    options
        .packages
        .iter()
        .map(|package| {
            info!("Processing package {:?}", package);

            let links = get_paths::<_, &Path>(package, options.target.as_ref(), options.dotfiles, true)?;

            for link in links {
                debug!("{:?}", link);
                if link.source.is_dir() {
                    if link.source.read_dir()?.next().is_none() {
                        info!("Directory {:?} is empty, removing...", link.source);
                        if !options.dry_run {
                            let res = remove_dir(link.source);
                            debug!("remove_dir result {:?}", res);
                        }
                    }
                } else if link.source.is_symlink()
                    && link.source.read_link()? == link.target
                {
                    info!("Removing link: {:?} -> {:?}", link.source, link.target);
                    if !options.dry_run {
                        let res = remove_file(link.source);
                        debug!("remove_file result {:?}", res);
                    }
                }
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

            let links = get_paths::<_, &Path>(package, options.target.as_ref(), options.dotfiles, false)?;

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
                        if link.source.is_symlink()
                            && link.source.read_link()? == link.target
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

fn get_paths<P, T>(package: P, source: T, map_dots: bool, uninstall: bool) -> Result<Vec<Link>>
where
    P: AsRef<Path>,
    T: AsRef<Path>,
{
    let mut links = Vec::new();

    for res in WalkDir::new(package)
        .min_depth(1)
        .contents_first(uninstall)
        .into_iter()
    {
        match res {
            Err(e) => return Err(e.into()),
            Ok(entry) => {
                let mut comp = entry.path().components();

                // Remove the current dir from the path
                comp.next();

                let raw_source = source.as_ref().join(comp.as_path());
                let mapped_source = match map_dots {
                    true => map_path_dots(raw_source),
                    false => raw_source,
                };

                // TODO: Should this be done elsewhere since this should be common to the entire
                // package?
                // Perform expansions for the path
                let source = shellexpand::full(
                    mapped_source
                        .to_str()
                        .ok_or(anyhow::anyhow!("Could not convert source to str"))?,
                )?;

                links.push(Link {
                    target: entry.path().canonicalize()?.to_path_buf(),
                    source: source.into_owned().into(),
                });
            }
        }
    }

    Ok(links)
}
