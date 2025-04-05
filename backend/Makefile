refresh:
	cd migration && cargo run -- refresh
gen-entity:
	sea-orm-cli generate entity -u postgres://myuser:mypassword@localhost/talk_app -o src/domain/entity
seed:
	cargo run --bin seed
clean-db:
	cd migration && cargo run -- fresh
setup-db: clean-db seed