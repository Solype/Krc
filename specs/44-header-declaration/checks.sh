# #44: a function of another file, declared without a body.

links() { # links <exit code> <file.kr>...: compiled together with -x in a temporary folder, run
    local expected="$1" dir code
    shift
    dir="$(mktemp -d)"
    cp "$@" "$dir" || return 1
    if ! (cd "$dir" && "$COMPILER" -x "${@##*/}" > /dev/null 2> err); then
        echo "does not compile: $(tail -n 1 "$dir/err")"
        rm -rf "$dir"
        return 1
    fi
    "$dir/a.out" > /dev/null
    code=$?
    rm -rf "$dir"
    [ "$code" -eq "$expected" ] || { echo "exit $code, expected $expected"; return 1; }
}

check "call_through_header" links 42 cases/lib.kr cases/prog.kr
check "header_and_definition" rejects cases/header_and_definition.kr "already defined"
manual "Decide whether a header and its definition may be in one file (refused for now)"
