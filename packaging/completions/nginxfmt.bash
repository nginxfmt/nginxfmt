_nginxfmt() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="nginxfmt"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        nginxfmt)
            opts="-w -h -V --write --check --config --tabs --spaces --indent-width --brace-style --max-blank-lines --trailing-newline --no-trailing-newline --preserve-inline-comments --no-preserve-inline-comments --generate-completions --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --indent-width)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --brace-style)
                    COMPREPLY=($(compgen -W "same_line next_line" -- "${cur}"))
                    return 0
                    ;;
                --max-blank-lines)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --generate-completions)
                    COMPREPLY=($(compgen -W "bash fish zsh" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _nginxfmt -o nosort -o bashdefault -o default nginxfmt
else
    complete -F _nginxfmt -o bashdefault -o default nginxfmt
fi
