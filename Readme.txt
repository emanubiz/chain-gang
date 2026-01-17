# PROGETTO: CHAIN GANG

## DESCRIZIONE GLOBALE DEL GIOCO

CHAIN GANG è uno sparatutto in prima persona (FPS) tattico a squadre (es. 3v3) cross-platform (Web & Mobile) con un'estetica Voxel/low-poly. Il suo core loop combina il gaming competitivo con la DeFi: le squadre bloccano una posta in gioco (in xDAI) su uno Smart Contract Gnosis prima di un match. Il gameplay è skill-based, con fuoco amico e fisica dei proiettili. Al termine del match, la squadra vincitrice riceve una prova crittografica dal server per sbloccare immediatamente e in modo trustless l'intero montepremi sul proprio wallet, eliminando intermediari.

## MASTER PLAN (Roadmap Globale)

Il progetto procederà per fasi "giocabili" per costruire gradualmente il gioco completo.

*   **FASE 1: Il Core del Gioco (Rust/Bevy)**
    *   **Obiettivo:** Avere 2 giocatori che si connettono, si vedono e si sparano in un ambiente di test. Focus sulle meccaniche di rete solide, senza grafica avanzata o blockchain in questa fase.
    *   **Focus Attuale.**

*   **FASE 2: L'Integrazione Web (WASM + React)**
    *   **Obiettivo:** Far girare il gioco dentro il browser (WASM) e creare l'interfaccia React attorno (Chat, Menu principale).

*   **FASE 3: L'Infrastruttura Multiplayer (Redis + Matchmaking)**
    *   **Obiettivo:** Gestire le room dinamiche e il matchmaking per assegnare i giocatori a stanze libere.

*   **FASE 4: La DeFi (Smart Contracts)**
    *   **Obiettivo:** Scrivere e testare i contratti di Escrow su Gnosis. Integrare il wallet login e le interazioni DeFi.

*   **FASE 5: Il "Glue" (Incollare tutto)**
    *   **Obiettivo:** Finalizzare l'integrazione tra server di gioco (firma della vittoria), frontend (sblocco dei fondi) e contratti blockchain.

## STRUTTURA DEL WORKSPACE (Monorepo)

chain-gang/
│
├── 📂 contracts/ (BLOCKCHAIN LAYER)
│ │ # Contiene gli Smart Contract Solidity (Gnosis)
│ ├── contracts/ # File .sol
│ ├── test/ # Test dei contratti
│ └── hardhat.config.js # O foundry.toml
│
├── 📂 web-portal/ (FRONTEND LAYER - React)
│ │ # Il sito web che l'utente visita
│ ├── src/ # Codice React, connessione Wallet, UI
│ ├── public/ # Qui verrà copiato il file .wasm del gioco
│ └── package.json # Dipendenze (Vite, Ethers, React)
│
├── 📂 game-engine/ (GAME LAYER - Rust Workspace)
│ │ # Il cuore del gioco (Client e Server)
│ ├── Cargo.toml # File che definisce il Workspace Rust
│ │
│ ├── 📦 game_shared/ # Logica condivisa (Protocollo, Costanti, Messaggi di rete)
│ │ └── src/lib.rs
│ │
│ ├── 📦 game_server/ # Il Server Autoritario (Linux Binary)
│ │ └── src/main.rs
│ │
│ └── 📦 game_client/ # Il Gioco Visuale (WASM target)
│ └── src/main.rs
│
└── 📂 infrastructure/ (OPS LAYER)
└── docker-compose.yml # Per lanciare Redis e Server locale velocemente

## ROADMAP LOCALE (Fase 1: Game Engine)

Questa è la roadmap specifica per il `game-engine`.

*   **Step 1.1 - Networking Skeleton:**
    *   Setup del workspace Rust.
    *   Server che ascolta su una porta.
    *   Client che si connette.
    *   Obiettivo: Verificare che `bevy_renet` funzioni e stabilire una connessione base.
    *   **STATO: COMPLETATO!** Abbiamo un server e un client che si connettono e scambiano eventi di connessione/disconnessione.

*   **Step 1.2 - Synchronized Physics (Il Cubo):**
    *   Aggiunta di `bevy_rapier3d`.
    *   Il server fa spawnare un cubo fisico.
    *   Il server invia la posizione e rotazione del cubo ai client.
    *   Il client vede il cubo muoversi e rimbalzare.
    *   **STATO: IN CORSO.** Stiamo implementando questa parte.

*   **Step 1.3 - Player Movement (Prediction):**
    *   Input del giocatore (WASD).
    *   Implementazione del sistema Client-Side Prediction.
    *   Obiettivo: Muoversi senza sentire lag.

*   **Step 1.4 - Voxel & Shooting:**
    *   Generazione di un pavimento di voxel statici.
    *   Logica dello sparo (Raycasting).
    *   Rimozione del voxel colpito sincronizzata tra tutti i client.

## DOVE SIAMO ORA (Punto Attuale)

Ci troviamo nello **Step 1.2 - Synchronized Physics (Il Cubo)** della Roadmap Locale del Game Engine.

*   **Abbiamo un server e un client che si connettono usando `bevy_renet`.** Il server rileva le nuove connessioni e disconnessioni.
*   **Stiamo lavorando per aggiungere `bevy_rapier3d`** per gestire la fisica.
*   **Il prossimo obiettivo immediato è compilare correttamente il progetto** dopo l'ultima serie di modifiche ai `Cargo.toml` e a `game_shared/src/lib.rs` per risolvere i problemi di `bevy_renet::renet::Channel` e le feature di `bevy_rapier3d`.
*   **Una volta compilato, l'aspettativa è vedere un cubo che cade e rimbalza sul pavimento nella finestra del client, sincronizzato dal server.**