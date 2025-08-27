
# Check if the "l" flag is provided
if [[ "$1" == "l" ]]; then
    # Skip build, run tests only
    (cd tests && clear && cargo test)
else
    # Run build first, then tests
    ./build.sh && (cd tests && clear && cargo test)
fi