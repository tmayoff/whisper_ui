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
      whisper_ui = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        strictDeps = true;

        #fixes issues related to openssl
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

        nativeBuildInputs = with pkgs; [
          libclang
        ];

        buildInputs = with pkgs;
          [
            pkg-config
            openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

        # Additional environment variables can be set directly
        # MY_CUSTOM_VAR = "some value";
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

        # Additional dev-shell environment variables can be set directly
        # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

        # Extra inputs can be added here; cargo and rustc are provided by default.
        packages = [
          # pkgs.ripgrep
        ];
      };
    });
}
