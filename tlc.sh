#!/bin/sh
# Locate tlc and run, check, or write a debug binary.
# Usage: tlc.sh run|check|build-debug <file.tlc> [output]
#
# The binary is lsp.tlc.binary.path in Zed settings, then <root>/build/tlc, then tlc on PATH.
# languages/tlc/tasks.json embeds this file as its sh -c script. Keep those copies identical.
set -eu

mode=${1:?}
file=${2:?}
out=${3:-}

root=${ZED_WORKTREE_ROOT:-}

configured_tlc() {
    root=$1
    config_home=${XDG_CONFIG_HOME:-$HOME/.config}
    user=$config_home/zed/settings.json
    [ -f "$user" ] || user=
    project=
    if [ -n "$root" ] && [ -f "$root/.zed/settings.json" ]; then
        project=$root/.zed/settings.json
    fi
    awk -v user="$user" -v project="$project" -v root="$root" -v home="${HOME:-}" '
function skipws() {
    while (P <= length(TEXT) && index(" \t\r\n", substr(TEXT, P, 1))) P++
}

function hexdigit(c) {
    n = index("0123456789abcdef", tolower(c))
    if (n == 0) return -1
    return n - 1
}

function parse_string(    c, hex, value, d, i) {
    P++
    S = ""
    while (P <= length(TEXT)) {
        c = substr(TEXT, P, 1)
        if (c == "\\") {
            P++
            if (P > length(TEXT)) break
            c = substr(TEXT, P, 1)
            if (c == "n") S = S "\n"
            else if (c == "t") S = S "\t"
            else if (c == "r") S = S "\r"
            else if (c == "b") S = S "\b"
            else if (c == "f") S = S "\f"
            else if (c == "u") {
                hex = substr(TEXT, P + 1, 4)
                value = 0
                for (i = 1; i <= 4; i++) {
                    d = hexdigit(substr(hex, i, 1))
                    if (d < 0) { value = -1; break }
                    value = value * 16 + d
                }
                if (value >= 0 && value <= 255) S = S sprintf("%c", value)
                P += 4
            } else S = S c
            P++
            continue
        }
        if (c == "\"") { P++; return }
        S = S c
        P++
    }
}

function skip_value(    c, depth) {
    skipws()
    c = substr(TEXT, P, 1)
    if (c == "\"") { parse_string(); return }
    if (c != "{" && c != "[") {
        while (P <= length(TEXT) && index(",}] \t\r\n", substr(TEXT, P, 1)) == 0) P++
        return
    }
    depth = 0
    while (P <= length(TEXT)) {
        c = substr(TEXT, P, 1)
        if (c == "\"") { parse_string(); continue }
        if (c == "{" || c == "[") depth++
        else if (c == "}" || c == "]") {
            depth--
            P++
            if (depth == 0) return
            continue
        }
        P++
    }
}

function object_get(key,    c, k) {
    skipws()
    if (substr(TEXT, P, 1) != "{") return 0
    P++
    while (1) {
        skipws()
        c = substr(TEXT, P, 1)
        if (c == "}" || c == "") { if (c == "}") P++; return 0 }
        if (c != "\"") return 0
        parse_string()
        k = S
        skipws()
        if (substr(TEXT, P, 1) != ":") return 0
        P++
        skipws()
        if (k == key) return 1
        skip_value()
        skipws()
        c = substr(TEXT, P, 1)
        if (c == ",") { P++; continue }
        if (c == "}") { P++; return 0 }
        return 0
    }
}

function strip_jsonc(text,    out, i, n, c, j, nxt) {
    out = ""
    i = 1
    n = length(text)
    while (i <= n) {
        c = substr(text, i, 1)
        if (c == "\"") {
            out = out c
            i++
            while (i <= n) {
                c = substr(text, i, 1)
                out = out c
                if (c == "\\") {
                    i++
                    if (i <= n) { out = out substr(text, i, 1); i++ }
                    continue
                }
                if (c == "\"") { i++; break }
                i++
            }
            continue
        }
        if (c == "/" && substr(text, i + 1, 1) == "/") {
            i += 2
            while (i <= n && substr(text, i, 1) != "\n") i++
            continue
        }
        if (c == "/" && substr(text, i + 1, 1) == "*") {
            i += 2
            while (i < n && !(substr(text, i, 1) == "*" && substr(text, i + 1, 1) == "/")) i++
            i += 2
            continue
        }
        out = out c
        i++
    }
    text = out
    out = ""
    i = 1
    n = length(text)
    while (i <= n) {
        c = substr(text, i, 1)
        if (c == "\"") {
            out = out c
            i++
            while (i <= n) {
                c = substr(text, i, 1)
                out = out c
                if (c == "\\") {
                    i++
                    if (i <= n) { out = out substr(text, i, 1); i++ }
                    continue
                }
                if (c == "\"") { i++; break }
                i++
            }
            continue
        }
        if (c == ",") {
            j = i + 1
            while (j <= n && index(" \t\r\n", substr(text, j, 1))) j++
            nxt = substr(text, j, 1)
            if (nxt == "}" || nxt == "]") { i++; continue }
        }
        out = out c
        i++
    }
    return out
}

function rtrim_slash(s) {
    while (length(s) > 1 && substr(s, length(s), 1) == "/") s = substr(s, 1, length(s) - 1)
    return s
}

function lookup(json) {
    TEXT = strip_jsonc(json)
    P = 1
    if (!object_get("lsp")) return ""
    if (!object_get("tlc")) return ""
    if (!object_get("binary")) return ""
    if (!object_get("path")) return ""
    skipws()
    if (substr(TEXT, P, 1) != "\"") return ""
    parse_string()
    return S
}

function readfile(path,    line, text) {
    text = ""
    while ((getline line < path) > 0) text = text line "\n"
    close(path)
    return text
}

function consider(path,    value) {
    if (path == "") return
    value = lookup(readfile(path))
    if (value != "" && value !~ /^[ \t]*$/) found = value
}

function resolve(path) {
    if (path == "~" || index(path, "~/") == 1) {
        if (home == "") return path
        home = rtrim_slash(home)
        if (path == "~") return home
        return home "/" substr(path, 3)
    }
    if (root != "" && substr(path, 1, 1) != "/" && index(path, "/") > 0) {
        root = rtrim_slash(root)
        if (root == "") return "/" path
        return root "/" path
    }
    return path
}

BEGIN {
    found = ""
    consider(user)
    consider(project)
    if (found == "") exit
    print resolve(found)
}
'
}

bin=$(configured_tlc "$root" || true)
if [ -n "$bin" ]; then
    case "$bin" in
    */*)
        if [ ! -f "$bin" ] || [ ! -x "$bin" ]; then
            echo "tlc from lsp.tlc.binary.path is not an executable file: $bin" >&2
            exit 1
        fi
        ;;
    *)
        resolved=$(command -v "$bin" || true)
        if [ -z "$resolved" ]; then
            echo "tlc from lsp.tlc.binary.path was not found on PATH: $bin" >&2
            exit 1
        fi
        bin=$resolved
        ;;
    esac
elif [ -n "$root" ] && [ -f "$root/build/tlc" ] && [ -x "$root/build/tlc" ]; then
    bin=$root/build/tlc
elif command -v tlc >/dev/null 2>&1; then
    bin=$(command -v tlc)
else
    echo "tlc was not found. Set lsp.tlc.binary.path in Zed settings, or build it with: cmake --build build --target tlc" >&2
    exit 1
fi

case "$mode" in
run)
    exec "$bin" --run "$file"
    ;;
check)
    exec "$bin" --check "$file"
    ;;
build-debug)
    if [ -z "$out" ]; then
        echo "build-debug needs an output path" >&2
        exit 2
    fi
    mkdir -p "$(dirname "$out")"
    exec "$bin" -g -o "$out" "$file"
    ;;
*)
    echo "unknown mode: $mode" >&2
    exit 2
    ;;
esac
