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