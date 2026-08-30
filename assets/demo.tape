# Regenerate assets/demo.gif with: vhs assets/demo.tape
# Requires: `cargo build --release` already run, and `vd` on PATH.

Output "assets/demo.gif"

Set Shell "bash"
Set FontSize 16
Set Width 1260
Set Height 780
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 55ms
Set WindowBar Colorful
Set BorderRadius 10

Env VDPM_HOME "/tmp/vdpm-demo-home"

Hide
Type "rm -rf $VDPM_HOME && mkdir -p $VDPM_HOME"
Enter
Type "alias vdpm=$PWD/target/release/vdpm"
Enter
Type "export PS1='$ '"
Enter
Type "clear"
Enter
Show

Type "vdpm list"
Enter
Sleep 1200ms

Type "vdpm install toml"
Enter
Sleep 1800ms

Type "vdpm list"
Enter
Sleep 1800ms

Type "vdpm enable toml"
Enter
Sleep 1500ms

Type "vdpm uninstall toml"
Enter
Sleep 1500ms

Type "vdpm list"
Enter
Sleep 2500ms
