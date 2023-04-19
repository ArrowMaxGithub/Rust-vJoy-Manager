set root=%cd%
chdir %root% 

rd target\release\assets /s /q
robocopy assets target/release/assets/ /e 
robocopy . target/release/ SDL2.dll

cargo build  --release  --features "profile"

cd ./target/release
start /b /wait hotas.exe
pause