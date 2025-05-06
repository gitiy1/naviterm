{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  makeWrapper,
  mpv,
  ...
}: let
  manifest = (lib.importTOML ./Cargo.toml).package;
in
  rustPlatform.buildRustPackage {
    pname = manifest.name;
    version = manifest.version;

    cargoLock.lockFile = ./Cargo.lock;
    src = lib.cleanSource ./.;

    nativeBuildInputs = [pkg-config makeWrapper];
    buildInputs = [openssl];

    postInstall = ''
      wrapProgram $out/bin/naviterm \
        --prefix PATH : ${mpv}/bin
    '';

    meta = with lib; {
      description = "Terminal user interface client for Navidrome written in Rust";
      homepage = "https://gitlab.com/detoxify92/naviterm";
      licenses = licenses.gpl3Plus;
      platforms = platforms.linux;
    };
  }
