#!/bin/bash
# The open issues of the repository, from GitHub through `gh`.
#
# Usage: scripts/get-issue.sh [-h|--help] [--gfi] [issue]
#   (no argument)  every open issue but the "good first issue" ones: number, title, labels
#   --gfi          the "good first issue" ones too
#   issue          the details of this issue: title, state, labels, body
# No environment variable; `gh` must be installed and logged in (gh auth login).
# Exit code: 0 on success, 1 when gh fails, 2 on a wrong argument.
# Example: scripts/get-issue.sh 2

if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
fi

gfi=0
issue=
for arg in "$@"; do
    case "$arg" in
        --gfi) gfi=1 ;;
        *[!0-9]* | '')
            echo "unknown argument: $arg (see --help)" >&2
            exit 2
            ;;
        *)
            if [ -n "$issue" ]; then
                echo "one issue at a time: $issue and $arg (see --help)" >&2
                exit 2
            fi
            issue="$arg"
            ;;
    esac
done

if [ -n "$issue" ]; then
    gh issue view "$issue" || exit 1
    exit 0
fi

gh issue list --state open --limit 500 --json number,title,labels --jq "
    .[] | [.labels[].name] as \$labels
    | select($gfi == 1 or (\$labels | index(\"good first issue\") | not))
    | \"#\(.number)\t\(.title)\t[\(\$labels | join(\", \"))]\"
" || exit 1
