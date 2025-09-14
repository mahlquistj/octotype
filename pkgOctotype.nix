{
  lib,
  makeRustPlatform,
  stdenv,
  pkg-config,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
  makeRustPlatform.buildRustPackage {
    inherit (cargoToml.workspace.package) version;
    name = "octotype";
    src = ./.;
    cargoLock.lockfile = ./Cargo.lock;

    nativeBuildInputs = lib.optionals stdenv.is_linux [
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
