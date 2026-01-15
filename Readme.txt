CHAIN GANG
Chain Gang e' uno sparatutto tattico Voxel (1-4 giocatori) con meccanica "Winner Takes All".
I giocatori puntano crypto (xDAI su Gnosis Chain), combattono nell'arena e il vincitore ottiene l'intero montepremi sbloccato crittograficamente dal server.
STACK TECNOLOGICO
Game Engine: Rust + Bevy (ECS) + Renet (Networking)
Frontend: React + Vite (TypeScript)
Blockchain: Hardhat (Solidity) + Gnosis Chain
STRUTTURA DEL WORKSPACE
Il progetto e' un Monorepo diviso in tre cartelle:
/game-engine
Il cuore del gioco. Contiene sia il Server Autoritativo (Linux) che il Client (WASM).
/web-portal
L'interfaccia React per collegare il Wallet e lanciare il gioco nel browser.
/contracts
Smart Contracts per la gestione dell'Escrow dei fondi su Gnosis Chain.
GUIDA RAPIDA (QUICK START)
A. Per avviare il Server di Gioco:
cd game-engine
cargo run --bin game_server
B. Per avviare il Client di Gioco (Nativo):
cd game-engine
cargo run --bin game_client
C. Per avviare il Frontend Web:
cd web-portal
npm install
npm run dev
STATO DEL PROGETTO
Pre-Alpha.
Struttura del workspace inizializzata.
Rendering Voxel base funzionante.
Setup iniziale del networking configurato.