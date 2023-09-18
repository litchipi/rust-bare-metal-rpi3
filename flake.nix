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
      libudev-zero
    ];

    mkScript = name: deps: text: let 
      app = pkgs.writeShellApplication {
        inherit name text;
        runtimeInputs = build_deps ++ deps;
      };
    in { type = "app"; program = "${app}/bin/${name}"; };

    config_file = builtins.concatStringsSep "\n" [
      "init_uart_clock=48000000"
      "arm_64bit=1"
    ];
    
    target_name = "jam_helper";
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = build_deps ++ [ pkgs.qemu pkgs.minicom ];
      PKG_CONFIG_PATH="${pkgs.libudev-zero}/lib/pkgconfig:$PKG_CONFIG_PATH";
    };
    apps.${system} = rec {
      default = emulate;

      build = mkScript "build" [] ''
        echo "[*] Building target"
        mkdir -p out target
        export "CARGO_TARGET_DIR"="$(realpath ./target)"
        cd execmain
        cargo build --target="aarch64-unknown-none-softfloat" --release
        cd ..
        cp ./target/${rust_target}/release/${target_name} out/kernel.elf
        aarch64-elf-strip out/kernel.elf
        aarch64-elf-objcopy -O binary out/kernel.elf out/kernel8.img
      '';
      
      emulate = mkScript "emulate" [ pkgs.qemu ] ''
        ${build.program}
        echo "[*] Starting emulation"
        qemu-system-aarch64 -M raspi3b -serial stdio -display none -kernel ./out/kernel8.img
      '';

      emulate-chainloader = let
        qemu_args = builtins.concatStringsSep " " [
          "-M raspi3b"
          "-serial pty"
          # "-monitor stdio"
          "-D ./chainloader.log"
          "-display none"
          "-kernel ./out/chainloader.img"
        ];
      in mkScript "emulate-chainloader" [ pkgs.qemu ] ''
        ${chainloader-client.program}
        echo "[*] Starting chainloader emulation"
        qemu-system-aarch64 ${qemu_args}
      '';

      provision_chainloader = mkScript "transfer" [ ] ''
        set -e
        if [ $# -lt 1 ]; then
          echo "Usage: $0 <copy destination>"
          exit 1;
        fi
        ${chainloader-client.program}
        echo "[*] Provisionning the chainloader"
        cp out/chainloader.img "$1/kernel8.img"
        cat << EOF > "$1/config.txt"
        ${config_file}
        EOF
        shift 1;
        sync
        echo "Done"
      '';

      chainloader-server = mkScript "chainloader-server" [] ''
        echo "[*] Starting the chainloader server"
        cd chainloader-server
        cargo b --release --target x86_64-unknown-linux-gnu
        sudo ../target/x86_64-unknown-linux-gnu/release/chainloader-server -s "$1" -k ../out/kernel8.img
      '';

      chainloader-client = mkScript "chainloader-client" [] ''
        echo "[*] Building the chainloader server"
        cd chainloader-client
        cargo build --target="aarch64-unknown-none-softfloat" --release
        mkdir -p ../out
        cd ..
        cp ./target/${rust_target}/release/chainloader-client ./out/chainloader.elf
        aarch64-elf-strip out/chainloader.elf
        aarch64-elf-objcopy -O binary out/chainloader.elf out/chainloader.img
      '';
    };
  };
}
