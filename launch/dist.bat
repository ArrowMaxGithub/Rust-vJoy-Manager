:: Setup to be called from the workspace root through a vscode task

rd target\dist\assets /s /q
robocopy assets target/dist/assets/ /e 
robocopy . target/dist/ SDL2.dll

cargo build --profile dist

pause