all: release

check:
	cargo check

debug:
	cargo build

release:
	cargo build --release

.PHONY: clean
clean:
	cargo clean

.PHONY: rsync
rsync:
	rsync -av --exclude=target ../kvm-box wjdev:/root/

