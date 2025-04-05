# talkapp

## マイレーションファイル作成方法

```bash
sea-orm-cli migrate generate create_table_{テーブル名}
```

## マイレーション実行方法

```bash
cd migration
cargo run -- up

cd migration
cargo run -- refresh
```

## マイレーションロールバック方法

```bash
cd migration
cargo run -- down
```

## entity作成方法

```bash
sea-orm-cli generate entity -u postgres://myuser:mypassword@localhost/talk_app -o src/domain/entity
```