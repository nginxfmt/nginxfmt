complete -c nginxfmt -l config -d 'Path to a nginxfmt config file' -r -F
complete -c nginxfmt -l indent-width -d 'Number of spaces per indentation level' -r
complete -c nginxfmt -l brace-style -d 'Brace placement style' -r -f -a "same_line\t''
next_line\t''"
complete -c nginxfmt -l max-blank-lines -d 'Maximum consecutive blank lines to preserve' -r
complete -c nginxfmt -l generate-completions -d 'Generate shell completions for the given shell' -r -f -a "bash\t''
fish\t''
zsh\t''"
complete -c nginxfmt -s w -l write -d 'Write formatted output back to the file'
complete -c nginxfmt -l check -d 'Check whether the file is formatted; exit with status 1 if not'
complete -c nginxfmt -l tabs -d 'Use tabs for indentation'
complete -c nginxfmt -l spaces -d 'Use spaces for indentation'
complete -c nginxfmt -l trailing-newline -d 'Ensure output ends with a trailing newline'
complete -c nginxfmt -l no-trailing-newline -d 'Omit the trailing newline at end of output'
complete -c nginxfmt -l preserve-inline-comments -d 'Preserve inline comments on the same line'
complete -c nginxfmt -l no-preserve-inline-comments -d 'Strip inline comments instead of preserving them'
complete -c nginxfmt -s h -l help -d 'Print help'
complete -c nginxfmt -s V -l version -d 'Print version'
