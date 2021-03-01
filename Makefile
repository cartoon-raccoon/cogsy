CC=cargo
TARG=cogsy
clean:
	$(CC) build --release

install:
	$(CC) build --release
	sudo cp target/release/$(TARG) /usr/bin/$(TARG)