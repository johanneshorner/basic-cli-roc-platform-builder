{
  outputs =
    {
      flake-parts,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        flake-parts.flakeModules.modules
      ];

      systems = [
        "x86_64-linux"
      ];

      perSystem =
        {
          pkgs,
          ...
        }:
        let
          # `musl-gcc` from `pkgs.musl.dev` doesn't resolve to the correct paths with
          # `-print-file-name` so we use `pkgsMusl.stdenv.cc`. To avoid shadowing or host `cc`
          # which is required for the build script we wrap the musl compiler with musl- prefix to
          # avoid shadowing the hosts `cc`.
          musl-gcc = pkgs.runCommand "musl-gcc" { } ''
            mkdir -p $out/bin
            for bin in ${pkgs.pkgsMusl.stdenv.cc}/bin/*; do
              ln -s "$bin" "$out/bin/musl-$(basename "$bin")"
            done
          '';
        in
        {
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              just
              musl-gcc
            ];
          };
        };
    };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };
}
