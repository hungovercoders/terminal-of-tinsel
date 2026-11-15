# Played by docs/demos/render.sh, which draws it with the theme in theme.json.
# Written in the language of VHS, but not played by it: see render.sh.
Set FontSize 15
Set Width 1000
Set Height 380
Set Padding 24
Set TypingSpeed 60ms

# With seed 25 the answers are 4, 2 and 2. The second is answered wrongly
# on purpose, to show the explanation. The answers move when the roster
# changes: check them with `quiz -n 3 --json` before re-recording.
Sleep 400ms
Type "terminal-of-tinsel quiz -n 3"
Sleep 300ms
Enter
Sleep 2.5s
Type "4"
Sleep 3s
Enter
Sleep 2.5s
Type "1"
Sleep 3.5s
Enter
Sleep 2.5s
Type "2"
Sleep 3s
Enter
Sleep 3.5s
Type "x"
Sleep 2s
