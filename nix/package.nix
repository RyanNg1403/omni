{
  lib,
  rustPlatform,
  callPackage,
  runCommand,
  zig_0_15,
  zstd,
  pkg-config,
  git,
}:

let
  manifest = lib.importTOML ../Cargo.toml;
  zigDeps = callPackage ../vendor/libghostty-vt/build.zig.zon.nix {
    name = "omni-libghostty-vt-zig-cache";
    inherit zstd;
    linkFarm = name: entries:
      runCommand name { } ''
        mkdir -p $out
        ${lib.concatMapStringsSep "\n" (entry: ''
          cp -rL ${entry.path} $out/${entry.name}
        '') entries}
      '';
  };
in
rustPlatform.buildRustPackage {
  pname = "omni";
  version = manifest.package.version;

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ./..)) (
      lib.fileset.unions [
        ../assets
        ../src
        ../vendor/libghostty-vt
        ../vendor/libghostty-vt.vendor.json
        ../build.rs
        ../Cargo.lock
        ../Cargo.toml
      ]
    );
  };

  cargoHash = "sha256-yRT31RnfjSQy5bxFXVvM9zRM59WAPrBozu3S2tag6s8=";
  cargoDepsName = "omni";

  nativeBuildInputs = [
    git
    pkg-config
  ];

  env = {
    LIBGHOSTTY_VT_OPTIMIZE = "ReleaseFast";
    LIBGHOSTTY_VT_SIMD = "true";
    LIBGHOSTTY_VT_ZIG_SYSTEM_DIR = zigDeps;
    ZIG = lib.getExe zig_0_15;
  };

  preBuild = ''
    export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-global-cache"
    export ZIG_LOCAL_CACHE_DIR="$TMPDIR/zig-local-cache"
  '';

  # Rust tests are covered by the normal CI workflow. The Nix check is
  # intentionally build-only so it validates packaging inputs without
  # duplicating the full Rust test suite.
  doCheck = false;

  meta = {
    description = "Terminal workspace manager for AI coding agents";
    homepage = "https://github.com/RyanNg1403/omni";
    license = lib.licenses.agpl3Plus;
    mainProgram = "omni";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
}
