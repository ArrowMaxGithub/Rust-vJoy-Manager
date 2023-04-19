set root=%cd%
chdir %root% 

rd target\dist\assets /s /q
robocopy assets target/dist/assets/ /e 
robocopy . target/dist/ SDL2.dll

cargo build --profile dist

pause