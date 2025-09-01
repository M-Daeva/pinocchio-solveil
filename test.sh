# 1. build and test:                        ./test.sh
# 2. build and test with "show-output":     ./test.sh s
# 3. test:                                  ./test.sh l
# 4. test with "show-output":               ./test.sh ls

if [[ "$1" == "l" ]]; then
    (cd tests && clear && cargo test)
elif [[ "$1" == "ls" ]]; then
    (cd tests && clear && cargo show)
elif [[ "$1" == "s" ]]; then
    ./build.sh && (cd tests && clear && cargo show)
else
    ./build.sh && (cd tests && clear && cargo test)
fi
