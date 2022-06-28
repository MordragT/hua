debug := "target/debug/hua"

build:
    cargo build

setcap: build
    sudo setcap 'cap_dac_read_search=p cap_dac_override=p cap_chown=p' {{debug}}

run ARG: build
    {{debug}} {{ARG}}

cap-run ARG: build setcap
    {{debug}} {{ARG}}

sudo-run ARG: build
    sudo {{debug}} {{ARG}}

hua-clean: build setcap
    sudo rm -rf /hua
    {{debug}} init

hua-cache: build setcap
    {{debug}} cache add http://localhost:8080

hua-add: hua-clean hua-cache
    {{debug}} add recipes/make-bin.toml

