{
  description = "Build a cargo project without extra checks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      craneLib = crane.lib.${system};
      whisper_ui = craneLib.buildPackage rec {
        pname = "whisper";
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        strictDeps = true;

        #fixes issues related to openssl
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib/";
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

        nativeBuildInputs = with pkgs; [
          makeWrapper
          cmake
          pkg-config
        ];

        buildInputs = with pkgs; [
          libxkbcommon
          wayland
          openssl
        ];

        postInstall = ''
          wrapProgram $out/bin/${pname} \
            --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildInputs}
        '';

        # ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
        #   # Additional darwin specific inputs can be set here
        #   pkgs.libiconv
        # ];
      };
    in {
      checks = {
        inherit whisper_ui;
      };

      packages.default = whisper_ui;

      apps.default = flake-utils.lib.mkApp {
        drv = whisper_ui;
      };

      devShells.default = craneLib.devShell {
        # Inherit inputs from checks.
        checks = self.checks.${system};

        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib/";
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath whisper_ui.buildInputs}";
        packages = [];
      };
    });
}
