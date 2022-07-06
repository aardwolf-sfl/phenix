MAKEFILE_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
REPO_ROOT_DIR:=$(realpath ${MAKEFILE_DIR}/../../..)
TESTS_ROOT_DIR:=$(realpath ${MAKEFILE_DIR}/..)

generate-phenix:
	cd ${REPO_ROOT_DIR}/phenix-bin ; cargo run -- --language c ${TESTS_ROOT_DIR}/schema/index.phenix > ${MAKEFILE_DIR}/generated.c

build:
	cd ${REPO_ROOT_DIR}/phenix-capi ; cargo build
	$(CC) -Wall -Wextra -pedantic main.c ${REPO_ROOT_DIR}/target/debug/libphenix.a -o run

clean:
	$(RM) run generated.c
