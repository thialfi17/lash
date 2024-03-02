# Lash

Lash is a commandline application that is intended to replace GNU Stow. It is not intended to
be a drop-in replacement but it does aim to replace most of the implemented functionality of
GNU Stow. To see the key differences refer to [GNU Stow Differences](crate#compared-to-gnu-stow)

# Glossary

- _package_: A directory containing a file structure that you wish to install or manage
elsewhere on the filesystem - potentially mixed in with other files e.g. in your config
directory or `/usr/bin`

- _target directory_: The base directory to install the files from the package in. The
structure inside the package will be mirrored in the _target_ directory and symbolic links will
be made to connect the files.

# Configuration

Lash can be configured in several different ways. Global defaults can be configured with a
configuration file in the user's configuration directory. This respects the XDG User Dirs
specification. For workarea specifc configuration options a config file can be created in each
workarea.

Configuration files should be called `lash.toml` and are (predictably) in the TOML format.
Options are specified in the global namespace. No package specific options can be configured.
To see the supported options in the configuration file see [Config](src/config.rs)

Most options can also be specified on the commandline.

# Compared to GNU Stow

- Configured by TOML files called `lash.toml`
- Commandline arguments do not match (`lash link` vs `stow -S`)
- Does not fold any of the directory structure (lash always creates the folders and links
invididual files rather than attempting to minimize the number of links created).
- The `--dotfiles` option has been fixed. None of the bugs that plague GNU Stow are a problem
in this implementation. I am aware some fixes had been made and are available in patches but
even then some bugs remained (try using `--dotfiles` and `--adopt` with GNU Stow and the
patches!).
