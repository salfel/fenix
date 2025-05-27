{
  description = "Fenix operating system";

  inputs = {nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";};

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [gcc-arm-embedded cargo qemu rustup];
      shellHook = ''
        TARGET="armv7a-none-eabi"

        if ! rustup target list --installed | grep -q $TARGET; then
          rustup target add $TARGET
        fi

        # Just a convenience for myself
        zsh
      '';
    };
  };
}
