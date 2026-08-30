# Regenerate assets/demo.gif with: vhs assets/demo.tape
# Requires: `cargo build --release` already run, `vd` on PATH, gcsfs
# installed into VisiData's own environment, and gcloud Application
# Default Credentials with access to gs://test-bucket-vdpm.

Output "assets/demo.gif"

Set Shell "bash"
Set FontSize 16
Set Width 1260
Set Height 900
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 45ms
Set WindowBar Colorful
Set BorderRadius 10

Hide
Type "DEMO_HOME=/tmp/vdpm-demo-home && rm -rf $DEMO_HOME && mkdir -p $DEMO_HOME/.config && cp -r ~/.config/gcloud $DEMO_HOME/.config/gcloud && export HOME=$DEMO_HOME && alias vdpm=$PWD/target/release/vdpm && export PS1='$ ' && clear"
Enter
Show

Type "vdpm install gcs --source https://raw.githubusercontent.com/berkodeon/visidata-plugins/refs/heads/main/gcs.py"
Enter
Sleep 2000ms

Type "vd gs://test-bucket-vdpm/"
Enter
Sleep 1500ms

Type "vdpm enable gcs"
Enter
Sleep 1500ms

Type "vd gs://test-bucket-vdpm/"
Enter
Sleep 4000ms
Type "q"
Sleep 1000ms

Hide
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
