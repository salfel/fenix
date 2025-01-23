{
  description = "Fenix operating system";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable"; };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [ gcc-arm-embedded cargo qemu parted ];
        shellHook = ''
          zsh
        '';
      };

    };
}
