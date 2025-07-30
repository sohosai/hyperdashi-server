{
  description = "HyperDashi Backend Server Development Environment";

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

        # Rust toolchain specification
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" "rust-analyzer" ];
        };

        # Node.js for frontend development
        nodejs = pkgs.nodejs_20;

        # Database and development tools
        developmentTools = with pkgs; [
          # Database tools
          sqlite
          sqlx-cli
          
          # Development utilities
          git
          curl
          jq
          
          # Text editors and IDE support
          vim
          nano
          
          # Build tools
          pkg-config
          openssl
          
          # Optional: GUI tools if available
          dbeaver-bin
        ];

        # Runtime dependencies
        runtimeDeps = with pkgs; [
          # SSL/TLS support
          openssl
          
          # SQLite runtime
          sqlite
          
          # Network tools for testing
          netcat-gnu
        ];

        # Development shell packages
        shellPackages = [
          rustToolchain
          nodejs
          pkgs.yarn
          pkgs.pnpm
        ] ++ developmentTools ++ runtimeDeps;

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = shellPackages;

          # Environment variables
          shellHook = ''
            echo "🦀 HyperDashi Development Environment"
            echo "=================================================="
            echo "Rust version: $(rustc --version)"
            echo "Node.js version: $(node --version)"
            echo "SQLite version: $(sqlite3 --version)"
            echo "=================================================="
            
            # Set environment variables
            export DATABASE_URL="sqlite://hyperdashi.db"
            export RUST_LOG="debug"
            export RUST_BACKTRACE=1
            
            # Server configuration
            export SERVER_HOST="127.0.0.1"
            export SERVER_PORT="8081"
            
            # Storage configuration
            export STORAGE_TYPE="local"
            export STORAGE_MAX_FILE_SIZE_MB="10"
            export LOCAL_STORAGE_PATH="./uploads"
            
            # Create uploads directory if it doesn't exist
            mkdir -p uploads
            
            echo "Environment variables set:"
            echo "  DATABASE_URL: $DATABASE_URL"
            echo "  SERVER_PORT: $SERVER_PORT"
            echo "  STORAGE_MAX_FILE_SIZE_MB: $STORAGE_MAX_FILE_SIZE_MB"
            echo ""
            echo "Available commands:"
            echo "  cargo build         - Build the project"
            echo "  cargo run           - Run the development server"
            echo "  cargo test          - Run tests"
            echo "  sqlx migrate run    - Run database migrations"
            echo "  nix run .#setup-db  - Initial database setup"
            echo "  nix run .#dev       - Start development server"
            echo "  nix run .#test      - Run all tests"
            echo ""
          '';

          # Additional environment variables for development
          DATABASE_URL = "sqlite://hyperdashi.db";
          RUST_LOG = "debug";
          RUST_BACKTRACE = "1";
          
          # PKG_CONFIG_PATH for OpenSSL
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          
          # Library paths
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.openssl
            pkgs.sqlite
          ];
        };

        # Package outputs
        packages = {
          # Backend binary
          hyperdashi-server = pkgs.rustPlatform.buildRustPackage {
            pname = "hyperdashi-server";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustToolchain
            ];

            buildInputs = with pkgs; [
              openssl
              sqlite
            ];

            # Skip tests during build (can be run separately)
            doCheck = false;

            meta = with pkgs.lib; {
              description = "HyperDashi equipment management system backend";
              license = licenses.mit;
              maintainers = [ ];
            };
          };

          # Docker image
          docker-image = pkgs.dockerTools.buildImage {
            name = "hyperdashi-server";
            tag = "latest";

            contents = [
              self.packages.${system}.hyperdashi-server
              pkgs.sqlite
              pkgs.openssl
            ];

            config = {
              Cmd = [ "${self.packages.${system}.hyperdashi-server}/bin/hyperdashi-server" ];
              Env = [
                "DATABASE_URL=sqlite:///data/hyperdashi.db"
                "SERVER_HOST=0.0.0.0"
                "SERVER_PORT=8080"
                "STORAGE_TYPE=local"
                "LOCAL_STORAGE_PATH=/uploads"
              ];
              ExposedPorts = {
                "8080/tcp" = {};
              };
              Volumes = {
                "/data" = {};
                "/uploads" = {};
              };
            };
          };

          default = self.packages.${system}.hyperdashi-server;
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;

        # Apps for easy running
        apps = {
          # Run the server
          hyperdashi-server = flake-utils.lib.mkApp {
            drv = self.packages.${system}.hyperdashi-server;
          };

          # Development server with auto-reload
          dev = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "hyperdashi-dev" ''
              export DATABASE_URL="sqlite://hyperdashi.db"
              export RUST_LOG="debug"
              
              echo "Starting HyperDashi development server..."
              echo "Server will be available at http://localhost:8081"
              
              # Run migrations first
              sqlx migrate run
              
              # Start the server with cargo watch for auto-reload
              if command -v cargo-watch >/dev/null 2>&1; then
                cargo watch -x run
              else
                echo "cargo-watch not found, installing..."
                cargo install cargo-watch
                cargo watch -x run
              fi
            '';
          };

          # Database setup
          setup-db = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "setup-db" ''
              export DATABASE_URL="sqlite://hyperdashi.db"
              
              echo "Setting up HyperDashi database..."
              
              # Create database file if it doesn't exist
              touch hyperdashi.db
              
              # Run migrations
              sqlx migrate run
              
              echo "Database setup complete!"
              echo "Database file: $(pwd)/hyperdashi.db"
            '';
          };

          # Run tests
          test = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "hyperdashi-test" ''
              export DATABASE_URL="sqlite://test.db"
              export RUST_LOG="info"
              
              echo "Running HyperDashi tests..."
              
              # Clean up any existing test database
              rm -f test.db
              
              # Run tests
              cargo test
              
              # Clean up test database
              rm -f test.db
              
              echo "Tests completed!"
            '';
          };

          # Bootstrap command for initial setup
          bootstrap = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "hyperdashi-bootstrap" ''
              echo "🚀 Bootstrapping HyperDashi Server Development Environment"
              echo ""
              
              # Check if we're in a nix develop shell
              if [ -z "$IN_NIX_SHELL" ]; then
                echo "📦 Entering development shell..."
                nix develop -c $0
                exit $?
              fi
              
              echo "✅ Development environment ready"
              echo ""
              
              # Check if Cargo.lock exists
              if [ ! -f "Cargo.lock" ]; then
                echo "🔒 Creating Cargo.lock..."
                cargo generate-lockfile
              fi
              
              echo "🗄️  Setting up SQLite database..."
              export DATABASE_URL="sqlite://hyperdashi.db"
              
              # Create database file if it doesn't exist
              touch hyperdashi.db
              
              # Check if sqlx-cli is installed
              if ! command -v sqlx &> /dev/null; then
                echo "📦 Installing sqlx-cli..."
                cargo install sqlx-cli --no-default-features --features native-tls,sqlite
              fi
              
              # Run migrations
              sqlx migrate run
              
              echo ""
              echo "📁 Creating upload directory for local storage..."
              mkdir -p ./uploads
              
              echo ""
              echo "🔍 Checking dependencies..."
              cargo check
              
              echo ""
              echo "✨ Bootstrap complete! You can now:"
              echo ""
              echo "  1. Run development server:   nix run .#dev"
              echo "  2. Run tests:               nix run .#test"
              echo "  3. Or use cargo directly:    cargo run"
              echo ""
              echo "Default configuration:"
              echo "  - Database: SQLite (./hyperdashi.db)"
              echo "  - Server:   http://localhost:8081"
              echo "  - Storage:  Local filesystem (./uploads)"
            '';
          };

          default = self.apps.${system}.hyperdashi-server;
        };
      }
    );
}