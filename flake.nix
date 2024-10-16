{
	description = "lash: a modern replacement for GNU Stow";

	inputs = {
		nixpkgs.url = "github:NixOs/nixpkgs/nixpkgs-unstable";
		naersk.url = "github:nix-community/naersk";
		naersk.inputs.nixpkgs.follows = "nixpkgs";
		rust-overlay.url = "github:oxalica/rust-overlay";
		rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = {
		self,
		nixpkgs,
		rust-overlay,
		flake-utils,
		naersk,
		...
	}:
	flake-utils.lib.eachDefaultSystem (system:
	let
		overlays = [ (import rust-overlay) ];
		pkgs = import nixpkgs { inherit system overlays; };
		toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
		naersk' = pkgs.callPackage naersk {
			cargo = toolchain;
			rustc = toolchain;
		};
	in rec {
		defaultPackage = naersk'.buildPackage {
			src = ./.;
		};

		devShells.default = pkgs.mkShell {
			nativeBuildInputs = [
				toolchain
			];
		};
	});
}
