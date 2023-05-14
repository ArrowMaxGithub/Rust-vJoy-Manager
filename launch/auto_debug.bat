set root=%cd%
chdir %root% 

rd target\debug\assets /s /q
robocopy assets target/debug/assets/ /e 
robocopy . target/debug/ SDL2.dll

cargo build

cd ./target/debug
start /b /wait rust-vjoy-manager.exe
pause