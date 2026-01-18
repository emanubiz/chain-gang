@echo off
echo Avvio CHAIN GANG...
echo.

cd game-engine

echo Avvio Server...
start "CHAIN GANG - Server" cmd /k "cargo run --bin game_server"

timeout /t 2 /nobreak > nul

echo Avvio Client...
start "CHAIN GANG - Client" cmd /k "cargo run --bin game_client"

echo.
echo Server e Client avviati in finestre separate!
echo.
pause