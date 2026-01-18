# PROGETTO: CHAIN GANG

## DESCRIZIONE GLOBALE DEL GIOCO

CHAIN GANG è uno sparatutto in prima persona (FPS) tattico a squadre cross-platform (Web & Mobile) con un'estetica Voxel/low-poly. 

### Modalità di Gioco
*   **1v1 fino a 4v4** - Da duelli 1v1 a battaglie 4v4 a squadre
*   **Modalità Pratica** - Gioco libero senza scommesse per allenamento e divertimento
*   **Modalità Scommessa** - Match competitivi con posta in palio on-chain

### Meccanica DeFi (Modalità Scommessa)
Le squadre bloccano una posta in gioco (in xDAI) su uno Smart Contract Gnosis prima di un match. Il gameplay è skill-based, con fuoco amico e fisica dei proiettili. Al termine del match, la squadra vincitrice riceve una prova crittografica dal server per sbloccare immediatamente e in modo trustless l'intero montepremi sul proprio wallet, eliminando intermediari.

## MASTER PLAN (Roadmap Globale)

Il progetto procederà per fasi "giocabili" per costruire gradualmente il gioco completo.

*   **FASE 1: Il Core del Gioco (Rust/Bevy)**
    *   **Obiettivo:** Creare un FPS multiplayer funzionante con movimento, shooting e fisica.
    *   **Tecnologie:** Rust, Bevy Engine, bevy_renet (networking)
    *   **Stato:** IN CORSO - Step 1.3 completato ✅

*   **FASE 2: L'Integrazione Web (WASM + React)**
    *   **Obiettivo:** Far girare il gioco dentro il browser (WASM) e creare l'interfaccia React attorno (Chat, Menu principale).

*   **FASE 3: L'Infrastruttura Multiplayer (Redis + Matchmaking)**
    *   **Obiettivo:** Gestire le room dinamiche e il matchmaking per assegnare i giocatori a stanze libere.

*   **FASE 4: La DeFi (Smart Contracts)**
    *   **Obiettivo:** Scrivere e testare i contratti di Escrow su Gnosis. Integrare il wallet login e le interazioni DeFi.

*   **FASE 5: Il "Glue" (Incollare tutto)**
    *   **Obiettivo:** Finalizzare l'integrazione tra server di gioco (firma della vittoria), frontend (sblocco dei fondi) e contratti blockchain.

## STRUTTURA DEL WORKSPACE (Monorepo)

```
chain-gang/
│
├── 📂 contracts/              (BLOCKCHAIN LAYER)
│   │ # Smart Contract Solidity (Gnosis)
│   ├── contracts/             # File .sol
│   ├── test/                  # Test dei contratti
│   └── hardhat.config.js
│
├── 📂 web-portal/             (FRONTEND LAYER - React)
│   │ # Il sito web che l'utente visita
│   ├── src/                   # React, connessione Wallet, UI
│   ├── public/                # Qui verrà copiato il .wasm del gioco
│   └── package.json
│
├── 📂 game-engine/            (GAME LAYER - Rust Workspace)
│   │ # Il cuore del gioco (Client e Server)
│   ├── Cargo.toml             # Workspace Rust
│   │
│   ├── 📦 game_shared/        # Logica condivisa (Protocollo, Messaggi di rete)
│   │   └── src/lib.rs         # PlayerInput, NetworkMessage, apply_player_movement()
│   │
│   ├── 📦 game_server/        # Il Server Autoritativo (Linux Binary)
│   │   └── src/main.rs        # Gestione connessioni, fisica server-side, sync
│   │
│   └── 📦 game_client/        # Il Gioco Visuale (WASM target)
│       └── src/main.rs        # Client-side prediction, rendering, input
│
└── 📂 infrastructure/         (OPS LAYER)
    └── docker-compose.yml
```

## ROADMAP LOCALE (Fase 1: Game Engine)

### ✅ Step 1.1 - Networking Skeleton
*   Setup del workspace Rust (3 crate: shared, server, client)
*   Server che ascolta su porta 5000
*   Client che si connette
*   Scambio di eventi di connessione/disconnessione tramite `bevy_renet`
*   **STATO: COMPLETATO** ✅

### ✅ Step 1.2 - Synchronized Physics (Il Cubo)
*   Integrazione di fisica manuale (no bevy_rapier per semplicità)
*   Il server spawna un cubo che cade e rimbalza
*   Il server invia posizione/rotazione del cubo ai client tramite `NetworkMessage::RigidBodyUpdate`
*   Il client visualizza il cubo sincronizzato
*   Fix del `transport.send_packets()` per inviare effettivamente i pacchetti UDP
*   **STATO: COMPLETATO** ✅

### ✅ Step 1.3 - Player Movement (Client-Side Prediction)
*   **Input del giocatore:** WASD per muoversi, Spazio per saltare
*   **Client-Side Prediction:** Il movimento è applicato IMMEDIATAMENTE sul client per zero lag percepito
*   **Server Autoritativo:** Il server riceve gli input, li processa e invia lo stato aggiornato
*   **Reconciliation:** Il client corregge la sua posizione quando riceve aggiornamenti dal server
*   **Funzione condivisa:** `apply_player_movement()` usata sia da client che server per garantire coerenza
*   **Multi-player:** Spawn di giocatori multipli (verde per locale, rosso per remoti)
*   **STATO: COMPLETATO** ✅

### 🚧 Step 1.4 - Player Experience Refinement
*   **Obiettivo:** Migliorare la sensazione di gioco e la fluidità
    *   **Rotazione della camera** (mouse look) - Controllo First Person
    *   **Camera che segue il giocatore** - Vista FPS
    *   **Interpolazione** dei giocatori remoti per movimento fluido (no teleport)
    *   **Riapplicazione degli input pendenti** dopo reconciliation per prediction perfetta
*   **STATO: IN CORSO** 🎯

### 📋 Step 1.5 - Voxel & Shooting
*   Generazione di un ambiente voxel di base
*   Logica dello sparo (Raycasting)
*   Rimozione del voxel colpito sincronizzata tra tutti i client
*   Hit detection sui giocatori
*   Health system e respawn
*   **STATO: PIANIFICATO**

### 📋 Step 1.6 - Game Modes
*   Sistema di lobby per 1v1, 2v2, 3v3, 4v4
*   Matchmaking per modalità pratica
*   Timer di match e win conditions
*   Scoreboard e statistiche partita
*   **STATO: PIANIFICATO**

## TECNOLOGIE UTILIZZATE

### Game Engine
*   **Rust** - Linguaggio di programmazione (performance + safety)
*   **Bevy 0.14** - Game Engine ECS (Entity Component System)
*   **bevy_renet 0.0.12** - Networking library (client-server)
*   **bincode** - Serializzazione binaria per messaggi di rete
*   **serde** - Serializzazione/Deserializzazione

### Frontend (Futuro)
*   **React + TypeScript** - UI framework
*   **Vite** - Build tool
*   **Ethers.js** - Interazione con blockchain

### Blockchain (Futuro)
*   **Solidity** - Smart contracts
*   **Hardhat** - Development environment
*   **Gnosis Chain** - Network (xDAI)

## COME ESEGUIRE IL PROGETTO

### Prerequisiti
```bash
# Installa Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verifica installazione
rustc --version
cargo --version
```

### Esecuzione del Game Engine

```bash
# Terminal 1 - Avvia il Server
cd game-engine
cargo run --bin game_server

# Terminal 2 - Avvia il Client
cargo run --bin game_client

# (Opzionale) Terminal 3 - Secondo client per testare multiplayer
cargo run --bin game_client
```

### Controlli di Gioco
*   **W** - Avanti
*   **A** - Sinistra
*   **S** - Indietro
*   **D** - Destra
*   **Spazio** - Salto

## NOTE TECNICHE IMPORTANTI

### Architettura di Networking
*   **Client-Server Autoritativo:** Il server è l'unica fonte di verità
*   **Client-Side Prediction:** Il client predice il movimento per responsività immediata
*   **Server Reconciliation:** Quando il server invia lo stato aggiornato, il client corregge eventuali divergenze
*   **Sequence Numbers:** Ogni input ha un numero di sequenza per tracciare quale input il server ha processato

### Fisica
*   **Fisica custom:** Implementata manualmente (no bevy_rapier) per controllo totale e compatibilità WASM
*   **Gravità:** -9.81 m/s²
*   **Collisione pavimento:** Controllo semplice Y <= PLAYER_HEIGHT/2
*   **Movimento:** 5.0 m/s di velocità base
*   **Salto:** 5.0 m/s di forza verticale

### Messaggi di Rete
*   **PlayerInput:** Client → Server (input WASD + jump + sequence_number)
*   **PlayerStateUpdate:** Server → Client (posizione, velocità, rotazione + sequence_number)
*   **RigidBodyUpdate:** Server → Client (oggetti non-giocatore come il cubo)
*   **PlayerConnected/Disconnected:** Server → All Clients (notifiche)

## PROSSIMI PASSI (Immediati)

1. ✅ Risolvere errori di compilazione (`ClientId` vs `u64`)
2. 🎯 Implementare mouse look (rotazione camera)
3. 🎯 Camera che segue il giocatore
4. 🎯 Interpolazione dei giocatori remoti
5. 🎯 Riapplicazione input pendenti (reconciliation completa)

## STATO DEL PROGETTO

**Ultimo aggiornamento:** 18 Gennaio 2025  
**Fase corrente:** FASE 1 - Step 1.3 completato  
**Prossimo milestone:** Step 1.4 - Player Experience Refinement  
**Livello di completamento Fase 1:** ~60%

---

**Note:** Questo progetto è in sviluppo attivo. La documentazione viene aggiornata ad ogni step completato.