refresh:
	cd migrations && cargo run -- refresh
gen-entity:
	sea-orm-cli generate entity -u postgres://myuser:mypassword@localhost/talk_app -o src/domain/entity