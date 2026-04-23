Output demo.gif

Require ironsubst

Set Shell "zsh"
Set WindowBar Colorful
Set FontSize 20
Set Width 900
Set Height 600

Type `export WHAT_AM_I="cool"`
Enter
Sleep 150ms
Type `export PUTIN_KHUYLO="true"`
Enter
Sleep 150ms

Type "cat << 'EOF' > template.txt"
Sleep 150ms
Enter
Sleep 20ms
Type "Welcome ${USER:-Guest}!"
Sleep 150ms
Enter
Sleep 20ms
Type "Your shell is ${SHELL:-Unknown}."
Sleep 150ms
Enter
Sleep 20ms
Type "You are ${WHAT_AM_I}."
Sleep 150ms
Enter
Sleep 20ms
Type@4ms "${PUTIN_KHUYLO:+I agree that putin doesn't respect Ukrainian sovereignty and territorial integrity. More info: https://en.wikipedia.org/wiki/Putin_khuylo!}"
Sleep 150ms
Enter
Sleep 20ms
Type "${AWS_ACCESS_KEY_ID:+I am leaking my AWS_ACCESS_KEY_ID right now}"
Sleep 150ms
Enter
Sleep 20ms
Type "EOF"
Sleep 50ms
Enter
Sleep 500ms

Type "ironsubst -i template.txt"
Sleep 1s
Enter
Sleep 5s
Enter 4

Type "ironsubst -- '${NOT_SET:=fallback!}'"
Sleep 50ms
Enter
Sleep 2s

Type "ironsubst --require-values -- '${NOT_SET:=fallback!}'"
Sleep 50ms
Enter
Sleep 2s

Type "export EMPTY=''"
Sleep 25ms
Enter
Type "ironsubst --require-values -- '${EMPTY}'"
Sleep 50ms
Enter
Sleep 2s

Type "ironsubst --require-nonempty-values -- '${EMPTY}'"
Sleep 50ms
Enter
Sleep 5s
