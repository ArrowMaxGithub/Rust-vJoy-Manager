:: Setup to be called from the workspace root through a vscode task

rd target\release\assets /s /q
robocopy assets target/release/assets/ /e 
robocopy . target/release/ SDL2.dll

cargo build  --release  --features "profile"

cd ./target/release
start /b /wait rust-vjoy-manager.exe
pause