{
  description = "doc_man - 文書管理システム";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.rust-analyzer
          pkgs.rustfmt
          pkgs.clippy
          pkgs.trunk
          pkgs.wasm-bindgen-cli
          pkgs.sqlx-cli
          pkgs.postgresql
        ];

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        shellHook = ''
          export DATABASE_URL="postgres://doc_man:doc_man@localhost:5432/doc_man"

          # wasm32 target が未インストールなら追加
          if ! rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
            echo "Note: wasm32-unknown-unknown target is provided by nix rustc"
          fi
        '';
      };
    };
}
