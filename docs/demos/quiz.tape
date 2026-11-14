# Recorded by docs/demos/render.sh, which fills in the theme from theme.json.
# Do not run vhs on this file directly.
Output docs/demos/.frames/quiz/
Set Shell bash
Set FontSize 15
Set Width 1000
Set Height 560
Set Padding 24
Set TypingSpeed 60ms
Set Theme @theme.json
Set Framerate 50

# With seed 25 the answers are 2, 3 and 1. The second is answered wrongly
# on purpose, to show the explanation.
Sleep 400ms
Type "terminal-of-tinsel quiz -n 3"
Sleep 300ms
Enter
Sleep 2.5s
Type "2"
Sleep 3s
Enter
Sleep 2.5s
Type "1"
Sleep 3.5s
Enter
Sleep 2.5s
Type "1"
Sleep 3s
Enter
Sleep 3.5s
Type "x"
Sleep 2s
