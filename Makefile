CC=cargo
TARG=cogsy

build:
	$(CC) build --release --locked --target-dir target

install: build
	install -DmT755 target/release/$(TARG) /usr/bin/$(TARG)