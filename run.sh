#!/bin/bash

echo "Avvio CHAIN GANG..."
echo ""

cd game-engine

# Controlla quale terminale è disponibile
if command -v gnome-terminal &> /dev/null; then
    echo "Avvio Server..."
    gnome-terminal -- bash -c "cargo run --bin game_server; exec bash"
    
    sleep 2
    
    echo "Avvio Client..."
    gnome-terminal -- bash -c "cargo run --bin game_client; exec bash"
    
elif command -v xterm &> /dev/null; then
    echo "Avvio Server..."
    xterm -e "cargo run --bin game_server" &
    
    sleep 2
    
    echo "Avvio Client..."
    xterm -e "cargo run --bin game_client" &
    
elif command -v konsole &> /dev/null; then
    echo "Avvio Server..."
    konsole -e bash -c "cargo run --bin game_server; exec bash" &
    
    sleep 2
    
    echo "Avvio Client..."
    konsole -e bash -c "cargo run --bin game_client; exec bash" &
    
else
    echo "Nessun terminale compatibile trovato (gnome-terminal, xterm, konsole)"
    echo "Usa tmux invece:"
    echo "   tmux new-session -d 'cargo run --bin game_server'"
    echo "   tmux split-window -h 'cargo run --bin game_client'"
    echo "   tmux attach"
    exit 1
fi

echo ""
echo "Server e Client avviati in finestre separate!"