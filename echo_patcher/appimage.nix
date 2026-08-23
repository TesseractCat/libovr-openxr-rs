# Build with:
#   nix-build echo_patcher/appimage.nix
#
# The result is a Type 2 x86_64 AppImage containing echo_patcher.
{ pkgs ? import <nixpkgs> {} }:

let
  inherit (pkgs) lib;

  echoPatcher = pkgs.rustPlatform.buildRustPackage {
    pname = "echo_patcher";
    version = "0.3.0";
    src = lib.cleanSourceWith {
      src = ../.;
      filter = path: type:
        let base = baseNameOf path;
        in !(base == "target" || base == "artifacts" || base == ".git");
    };

    cargoLock = {
      lockFile = ../Cargo.lock;
      # Slint's dependency graph is locked in Cargo.lock.
      outputHashes = {};
    };

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [ pkgs.fontconfig ];
    cargoBuildFlags = [ "--bin" "echo_patcher" ];
    doCheck = false;

    installPhase = ''
      runHook preInstall
      install -Dm755 target/x86_64-unknown-linux-gnu/release/echo_patcher $out/bin/echo_patcher
      runHook postInstall
    '';
  };

  appDir = pkgs.runCommand "echo_patcher-AppDir" {} ''
    mkdir -p $out/usr/bin $out/usr/share/applications $out/usr/share/icons/hicolor/scalable/apps
    ln -s ${echoPatcher}/bin/echo_patcher $out/usr/bin/echo_patcher
    cp ${./echo_patcher.desktop} $out/usr/share/applications/echo_patcher.desktop
    cp ${./echo_patcher.svg} $out/usr/share/icons/hicolor/scalable/apps/echo_patcher.svg
    ln -s usr/bin/echo_patcher $out/AppRun
    ln -s usr/share/applications/echo_patcher.desktop $out/echo_patcher.desktop
    ln -s usr/share/icons/hicolor/scalable/apps/echo_patcher.svg $out/echo_patcher.svg
  '';

  # nixpkgs does not currently provide a usable appimagetool. This is the
  # pinned nix-bundle derivation described in the linked packaging article.
  nixBundle = pkgs.fetchFromGitHub {
    owner = "matthewbauer";
    repo = "nix-bundle";
    rev = "223f4ffc4179aa318c34dc873a08cb00090db829";
    sha256 = "0pqpx9vnjk9h24h9qlv4la76lh5ykljch6g487b26r1r2s9zg7kh";
  };
  # The article's 2021 derivation refers to stdenv.glibc, which current
  # nixpkgs exposes as pkgs.glibc instead.
  appimagetool = pkgs.callPackage "${nixBundle}/appimagetool.nix" {
    stdenv = pkgs.stdenv // { glibc = pkgs.glibc; };
  };
in
pkgs.stdenv.mkDerivation {
  pname = "echo_patcher";
  version = "0.3.0";
  src = appDir;
  nativeBuildInputs = [ appimagetool ];
  ARCH = "x86_64";

  unpackPhase = "true";
  buildPhase = ''
    cp -rL "$src" AppDir
    chmod -R u+w AppDir
  '';
  installPhase = ''
    mkdir -p $out
    appimagetool AppDir "$out/echo_patcher-x86_64.AppImage"
  '';
}
