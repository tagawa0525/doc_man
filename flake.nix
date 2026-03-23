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
          pkgs.lld
          pkgs.sqlx-cli
          pkgs.postgresql
          pkgs.just
          pkgs.podman-compose
        ];

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        shellHook = ''
          export PGDATA="$PWD/.pgdata"
          export PGHOST="$PWD/.pgdata"
          export PGPORT="5432"
          export PGDATABASE="doc_man"
          export PGUSER="doc_man"
          export DATABASE_URL="postgres://doc_man:doc_man@localhost:5432/doc_man"

          _dm_setup_db() {
            # 初期化
            if [ ! -d "$PGDATA" ]; then
              echo "==> PostgreSQL データディレクトリを初期化..."
              initdb --no-locale --encoding=UTF8 --auth=trust >/dev/null

              # Unix socket 認証 + パスワード認証設定
              cat >> "$PGDATA/postgresql.conf" <<CONF
          listen_addresses = 'localhost'
          port = $PGPORT
          unix_socket_directories = '$PGDATA'
          CONF
            fi

            # 起動（既に起動中なら何もしない）
            if ! pg_isready -h localhost -p "$PGPORT" -q 2>/dev/null; then
              echo "==> PostgreSQL を起動..."
              pg_ctl start -l "$PGDATA/server.log" -o "-k $PGDATA" -w >/dev/null
            fi

            # ユーザー作成（存在しなければ）
            if ! psql -h localhost -U "$USER" -d postgres -tAc "SELECT 1 FROM pg_roles WHERE rolname='doc_man'" 2>/dev/null | grep -q 1; then
              echo "==> ユーザー doc_man を作成..."
              createuser -h localhost -U "$USER" -s doc_man 2>/dev/null || true
              psql -h localhost -U "$USER" -d postgres -c "ALTER USER doc_man PASSWORD 'doc_man';" >/dev/null 2>&1
            fi

            # DB 作成（存在しなければ）
            if ! psql -h localhost -U doc_man -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='doc_man'" 2>/dev/null | grep -q 1; then
              echo "==> データベース doc_man を作成..."
              createdb -h localhost -U doc_man doc_man
            fi

            # マイグレーション
            echo "==> マイグレーションを実行..."
            sqlx migrate run --source server/migrations 2>&1 | grep -v "^Applied" || true
            echo "==> DB 準備完了"
          }

          _dm_setup_db

          echo ""
          echo "  just --list でコマンド一覧を表示"
          echo ""
        '';
      };
    };
}
