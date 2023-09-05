{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs: with inputs; let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ rust-overlay.overlays.default ];
    };
    target_name = "jam_helper";
    rust_target = "aarch64-unknown-none-softfloat";
    dev_env = {
      buildInputs = with pkgs; [
        (rust-bin.stable."1.72.0".default.override {
          extensions = [ "rust-src" ];
          targets = [ rust_target ];
        })
      ];
      RUSTFLAGS = builtins.concatStringsSep " " [
        "-C target-cpu=cortex-a53"
        "-C link-arg=--library-path=${./bsp_raspi3b1_2/src}"
        "-C link-arg=--script=${./bsp_raspi3b1_2/src/kernel.ld}"
        "-D warnings"
      ];
    };

    mkScript = name: text: let 
      app = pkgs.writeShellApplication {
        inherit name;
        text = ''
          export RUSTFLAGS="${dev_env.RUSTFLAGS}"

        '' + text;
        runtimeInputs = dev_env.buildInputs;
      };
    in { type = "app"; program = "${app}/bin/${name}"; };
    
  in {
    devShells.${system}.default = pkgs.mkShell dev_env;

    apps.${system} = rec {
      default = build;
      build = mkScript "build" ''
        env |grep RUSTFLAGS
        cargo build --target="aarch64-unknown-none-softfloat" --release
        mkdir -p target/out
        cp ./target/${rust_target}/release/${target_name} ./target/out/kernel8.elf
        # strip ./target/out/kernel8.elf
        # Objcopy
      '';
      emulate = mkScript "emulate" ''
      '';
      flash = mkScript "flash" ''
      '';
    };
  };
}
