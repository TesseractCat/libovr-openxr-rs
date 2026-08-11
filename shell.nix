# Development shell; intentionally uses classic nix-shell rather than flakes.
#
# Rustup is packaged by Nix because this project needs Rust's Windows stdlib,
# which is not normally shipped with the host-only nixpkgs rustc package. The
# exact Rust toolchain and target are pinned in rust-toolchain.toml.
{ pkgs ? import <nixpkgs> { config.allowUnfree = true; } }:

let
  mingw = pkgs.pkgsCross.mingwW64.stdenv.cc;
in
pkgs.mkShell {
  packages = [
    # Native Nix-built Rust runs host tests correctly on NixOS. Rustup remains
    # available only for the Windows GNU target pinned below.
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.clippy
    pkgs.rustup
    pkgs.pkg-config
    pkgs.gnumake
    pkgs.binutils
    pkgs.file
    pkgs.python3
    pkgs.monado
    pkgs.steam-run
    mingw
  ];

  # Cargo reads .cargo/config.toml for the target-specific linker. Keep rustup
  # and downloaded crates local to this checkout instead of altering the user's
  # global Rust installation.
  shellHook = ''
    export RUSTUP_HOME="$PWD/.rustup"
    export CARGO_HOME="$PWD/.cargo-home"
    # Ensure native test commands use Nix-built Rust, not Rustup's generic
    # Linux binaries. `tools/build-windows.sh` invokes Rustup explicitly.
    export PATH="${pkgs.cargo}/bin:${pkgs.rustc}/bin:${pkgs.rustfmt}/bin:${pkgs.clippy}/bin:$PATH"

    # Proton is deliberately not a Nix dependency: use the user's Steam-managed
    # build so the test prefix matches normal game execution.
    proton_root="$HOME/.local/share/Steam/steamapps/common"
    for candidate in "$proton_root/Proton 11.0/proton" \
                     "$proton_root/Proton - Experimental/proton" \
                     "$proton_root/Proton 10.0/proton"; do
      if [ -x "$candidate" ]; then
        export LIBOVR_OPENXR_PROTON="$candidate"
        break
      fi
    done

    echo "libovr-openxr-rs development shell"
    echo "  host checks: cargo test"
    echo "  Windows DLL: tools/build-windows.sh"
    echo "  Proton run: steam-run \"$LIBOVR_OPENXR_PROTON\" run <exe>"
    if [ -n "''${LIBOVR_OPENXR_PROTON:-}" ]; then
      echo "  Proton: $LIBOVR_OPENXR_PROTON"
    else
      echo "  Proton: not found under Steam's common directory"
    fi
  '';
}
