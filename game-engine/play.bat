@echo off
echo.
echo  ╔═══════════════════════════════╗
echo  ║     CHAIN GANG - Avvio        ║
echo  ╚═══════════════════════════════╝
echo.

echo [1/3] Avvio SERVER...
start "CHAIN GANG - Server" cmd /k "cargo run --bin game_server"

echo Attendo 8 secondi che il server si avvii...
timeout /t 8 /nobreak > nul

echo [2/3] Avvio CLIENT 1...
start "CHAIN GANG - Client 1" cmd /k "cargo run --bin game_client"

echo Attendo 3 secondi...
timeout /t 3 /nobreak > nul

echo [3/3] Avvio CLIENT 2...
start "CHAIN GANG - Client 2" cmd /k "cargo run --bin game_client"

echo.
echo Tutto avviato! Buona partita.
