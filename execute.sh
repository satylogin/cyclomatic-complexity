cargo install cargo-make

if [[ $# -eq 1 ]]; then
    cargo make --makefile DevWorkflow.toml $1
else
    echo "require one action to run. see cargo make --list-all-steps for all allowed actions."
fi