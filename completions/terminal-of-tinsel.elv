
set edit:completion:arg-completer[terminal-of-tinsel] = { |@words|
    var index = (count $words)
    set index = (- $index 1)

    put (env _CLAP_IFS="\n" _CLAP_COMPLETE_INDEX=(to-string $index) COMPLETE="elvish" terminal-of-tinsel -- $@words) | to-lines
}

