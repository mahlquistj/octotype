{
  rust-toolchain,
  lib,
  makeRustPlatform,
  stdenv,
  pkg-config,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  rustPlatform = makeRustPlatform {
    cargo = rust-toolchain;
    rustc = rust-toolchain;
  };
in
  rustPlatform.buildRustPackage {
    inherit (cargoToml.package) version;
    name = "octotype";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;

    nativeBuildInputs = lib.optionals stdenv.isLinux [
      pkg-config
    ];

    postInstall = ''
      install -D -m 644 misc/octotype.desktop -t $out/share/applications
      install -D -m 644 misc/logo.svg $out/share/icons/hicolor/scalable/apps/octotype.svg
    '';
    # Mac not supported for now.. TBD
    # + lib.optionalString stdenv.hostPlatform.isDarwin ''
    #   mkdir $out/Applications/
    #   mv misc/osx/OctoType.app/ $out/Applications/
    #   mkdir $out/Applications/OctoType.app/Contents/MacOS/
    #   ln -s $out/bin/octotype $out/Applications/Rio.app/Contents/MacOS/
    # '';

    meta = {
      description = "A typing trainer for your terminal!";
      homepage = "https://github.com/mahlquistj/octotype";
      license = lib.licenses.mit;
      platforms = lib.platforms.unix;
      mainProgram = "octotype";
    };
  }
