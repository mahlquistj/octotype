Output assets/demo.gif

Set Theme "Catppuccin Mocha"

Set TypingSpeed 115ms
Set Shell "nu"
Set FontSize 24
Set Width 1400
Set Height 1000

# Start program
Hide
Type "cargo run"
Enter 1
Wait+Screen /CTRL-Q/
Show
Sleep 1s

# Select Session
Enter 1
Sleep 1s

# Select Mode
Enter 1
Sleep 1s

# Select Source
Enter 1
Sleep 4s

# Session
Type "The quick brownfox"
Backspace 3
Type " fox jumps over the lazy dog, testing my typingspeed"
Backspace 5
Type " speed with every leap, but I'll soon catch up. "
Sleep 5s

# Return to menu
Escape 1
Sleep 500ms

# Open stats history
Down 1
Sleep 500ms
Enter 1
Sleep 2s
Down 1
Sleep 2s
Down 1
Sleep 2s

# View trends
Tab 1
Sleep 5s
