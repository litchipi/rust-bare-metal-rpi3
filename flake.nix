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
    rust_target = "aarch64-unknown-none-softfloat";
    rust_version = "1.72.0";
    build_deps = with pkgs; [
      (rust-bin.stable.${rust_version}.default.override {
        extensions = [ "rust-src" ];
        targets = [ rust_target ];
      })
      coreboot-toolchain.aarch64
    ];

    mkScript = name: deps: text: let 
      app = pkgs.writeShellApplication {
        inherit name text;
        runtimeInputs = build_deps ++ deps;
      };
    in { type = "app"; program = "${app}/bin/${name}"; };
    
    target_name = "jam_helper";
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = build_deps ++ [ pkgs.qemu ];
    };
    apps.${system} = rec {
      default = emulate;

      build = mkScript "build" [] ''
        cargo build --target="aarch64-unknown-none-softfloat" --release
        mkdir -p target/out
        cp ./target/${rust_target}/release/${target_name} target/out/kernel.elf
        cp target/out/kernel.elf target/out/kernel_stripped.elf
        # aarch64-elf-strip target/out/kernel_stripped.elf
        aarch64-elf-objcopy target/out/kernel_stripped.elf target/out/kernel.img
      '';
      
      emulate = mkScript "emulate" [ pkgs.qemu ] ''
        ${build.program}
        qemu-system-aarch64 -M raspi3b -serial stdio -display none -kernel ./target/out/kernel.img
      '';

      inspect = mkScript "inspect" [] ''
        aarch64-elf-readelf --headers ${build_kernel}/kernel.elf
      '';

      flash = mkScript "flash" [] ''
      '';
    };
  };
}
