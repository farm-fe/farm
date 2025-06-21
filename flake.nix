{
  description = "Farm - A fast build tool for web development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Get the specific Rust toolchain from rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Node.js and pnpm setup
        nodejs = pkgs.nodejs_22;
        pnpm = pkgs.pnpm;

        # Common development dependencies
        commonDevDeps = with pkgs; [
          # Rust toolchain
          rustToolchain
          pkg-config
          openssl

          # Node.js ecosystem
          nodejs
          pnpm
          typescript
          
          # Build tools
          git
          cargo-watch
          
          # Development utilities
          ripgrep
          fd
          jq
          
          # Cross-compilation support
          gcc
          
          # For native module compilation
          python3
        ];

        # Farm package
        farm = pkgs.rustPlatform.buildRustPackage rec {
          pname = "farm";
          version = "0.0.0";

          src = ./.;

          # Use the same Rust toolchain as specified in rust-toolchain.toml
          buildInputs = [ rustToolchain ];
          nativeBuildInputs = with pkgs; [ 
            pkg-config 
            nodejs
            pnpm
            python3
          ];

          # Install Node.js dependencies first
          preBuild = ''
            export HOME=$TMPDIR
            export npm_config_cache=$TMPDIR/.npm
            
            # Install Node.js dependencies
            pnpm install --frozen-lockfile
            
            # Build TypeScript packages
            pnpm run build || true
          '';

          # We need to provide a fake Cargo.lock hash since this is a workspace
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Build only the CLI and core packages
          buildPhase = ''
            runHook preBuild
            
            # Build Rust components
            cargo build --release --workspace
            
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            
            mkdir -p $out/bin
            
            # Install Rust binaries
            find target/release -maxdepth 1 -type f -executable | while read bin; do
              if [[ -f "$bin" && ! "$bin" =~ \.(so|dylib|dll)$ ]]; then
                cp "$bin" $out/bin/
              fi
            done
            
            # Install Node.js CLI if it exists
            if [ -d "packages/cli" ]; then
              mkdir -p $out/lib/farm
              cp -r packages/cli/* $out/lib/farm/
              
              # Create wrapper script for CLI
              cat > $out/bin/farm << 'EOF'
            #!/bin/sh
            exec ${nodejs}/bin/node $out/lib/farm/bin/farm.js "$@"
            EOF
              chmod +x $out/bin/farm
            fi
            
            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "A fast build tool for web development written in Rust";
            homepage = "https://farmfe.org";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

      in
      {
        packages = {
          default = farm;
          farm = farm;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = commonDevDeps;

          shellHook = ''
            echo "🌾 Farm development environment loaded"
            echo ""
            echo "Available commands:"
            echo "  pnpm bootstrap    - Install dependencies and build"
            echo "  pnpm start        - Start development server"
            echo "  pnpm start:rs     - Watch Rust changes"
            echo "  pnpm test         - Run tests"
            echo "  pnpm check        - Run linting"
            echo "  cargo build       - Build Rust components"
            echo "  cargo test        - Test Rust components"
            echo ""
            echo "Node.js: $(node --version)"
            echo "pnpm: $(pnpm --version)"
            echo "Rust: $(rustc --version)"
            echo ""
            
            # Set up environment variables
            export RUST_LOG=info
            export NODE_ENV=development
            
            # Ensure pnpm is available
            if ! command -v pnpm &> /dev/null; then
              echo "Installing pnpm globally..."
              npm install -g pnpm
            fi
          '';

          # Environment variables for development
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        };

        # Additional development shells for specific tasks
        devShells.rust-only = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            cargo-watch
          ];
          
          shellHook = ''
            echo "🦀 Rust-only development environment for Farm"
            echo "Rust: $(rustc --version)"
          '';
        };

        devShells.node-only = pkgs.mkShell {
          buildInputs = with pkgs; [
            nodejs
            pnpm
            typescript
          ];
          
          shellHook = ''
            echo "📦 Node.js-only development environment for Farm"
            echo "Node.js: $(node --version)"
            echo "pnpm: $(pnpm --version)"
          '';
        };
      });
}