# 🎮 CHAIN GANG

> **Tactical FPS meets DeFi** - Un gioco multiplayer competitivo dove ogni partita ha valore reale.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.14-blue.svg)](https://bevyengine.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## 🚀 Cos'è Chain Gang?

Chain Gang è uno **sparatutto tattico in prima persona** che unisce il gameplay competitivo skill-based con la blockchain. 

### 🎯 Modalità di Gioco

**📊 Flessibilità:** Da **1v1** a **4v4**
- 🥊 **1v1** - Duelli intensi uno contro uno
- 👥 **2v2** - Cooperazione a coppie
- 🎮 **3v3** - Team tactics classico
- 🏆 **4v4** - Battaglie a squadre complete

**🎲 Due Modalità:**

1. **⚡ Modalità Pratica** (Free Play)
   - Gioco libero senza scommesse
   - Perfetto per allenamento e divertimento
   - Matchmaking casual
   - Zero rischi, solo skill

2. **💰 Modalità Scommessa** (Ranked + DeFi)
   - Match competitivi con posta in palio
   - Le squadre bloccano xDAI su smart contract
   - I vincitori ricevono l'intero montepremi immediatamente
   - **Zero intermediari, 100% trustless**

### ✨ Feature Principali

- 🎯 **FPS Competitivo** - Gameplay skill-based con fisica dei proiettili e fuoco amico
- 👥 **1v1 fino a 4v4** - Modalità di gioco flessibili per ogni stile
- 🎲 **Doppia Modalità** - Pratica gratuita + Scommesse on-chain
- 🌐 **Cross-Platform** - Web (WASM) e Mobile
- 💰 **DeFi Integrato** - Smart contracts su Gnosis Chain per escrow trustless
- 🎨 **Estetica Voxel** - Stile low-poly moderno
- ⚡ **Client-Side Prediction** - Zero input lag percepito
- 🔒 **Server Autoritativo** - Anti-cheat nativo nell'architettura

## 📦 Struttura del Progetto

```
chain-gang/
├── 🎮 game-engine/        # Core del gioco (Rust + Bevy)
│   ├── game_shared/       # Logica condivisa client-server
│   ├── game_server/       # Server autoritativo
│   └── game_client/       # Client di gioco
├── ⚛️  web-portal/        # Frontend React + WASM
├── ⛓️  contracts/         # Smart contracts Solidity
└── 🐳 infrastructure/    # Docker & deployment
```

## 🎯 Stato del Progetto

**Fase Corrente:** FASE 1 - Core del Gioco

| Step | Descrizione | Stato |
|------|-------------|-------|
| 1.1 | Networking Skeleton | ✅ Completato |
| 1.2 | Synchronized Physics | ✅ Completato |
| 1.3 | Player Movement & Prediction | ✅ Completato |
| 1.4 | Player Experience Refinement | 🚧 In Corso |
| 1.5 | Voxel & Shooting | 📋 Pianificato |
| 1.6 | Game Modes & Lobby | 📋 Pianificato |

**Step 1.4 - Player Experience Refinement:**
- 🎯 Mouse look (rotazione camera)
- 🎯 Camera FPS che segue il giocatore
- 🎯 Interpolazione giocatori remoti (movimento fluido)
- 🎯 Reconciliation completa (riapplica input pendenti)

## 🚀 Quick Start

### Prerequisiti

```bash
# Installa Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verifica installazione
rustc --version
cargo --version
```

### Esecuzione

```bash
# Clona il repository
git clone https://github.com/yourusername/chain-gang.git
cd chain-gang/game-engine

# Terminal 1: Avvia il server
cargo run --bin game_server

# Terminal 2: Avvia il client
cargo run --bin game_client
```

### 🎮 Controlli

| Tasto | Azione |
|-------|--------|
| **W** | Avanti |
| **A** | Sinistra |
| **S** | Indietro |
| **D** | Destra |
| **Spazio** | Salto |

## 🎮 Come Funziona

### Modalità Pratica (Free Play)

```
1. 🔍 Entra in matchmaking
2. 🎯 Scegli la modalità (1v1, 2v2, 3v3, 4v4)
3. ⚔️  Gioca e divertiti
4. 📈 Migliora le tue skill
```

**Zero rischi, solo divertimento!**

### Modalità Scommessa (Ranked)

```
1. 💰 Le squadre bloccano xDAI su smart contract
   └─ Esempio: 4v4 → 10 xDAI per squadra = 20 xDAI pot

2. 🎮 Il match inizia (server autoritativo)
   └─ Gameplay skill-based con anti-cheat

3. 🏆 La squadra vincente riceve prova crittografica
   └─ Firma del server verificabile on-chain

4. ⚡ Sblocco immediato del premio (20 xDAI)
   └─ Trustless, nessun intermediario
   └─ Distribuito automaticamente ai membri del team
```

**100% Trustless. 100% Skill-Based.**

### Networking

Chain Gang utilizza un'architettura **client-server autoritativa** con **client-side prediction**:

```
Client                          Server
  │                               │
  ├─► PlayerInput (seq: 1) ──────►│
  │   (predice localmente)        │ Processa input
  │                               │ Simula fisica
  │◄── PlayerState (seq: 1) ──────┤ 
  │   (reconcilia se diverso)     │
  └─► PlayerInput (seq: 2) ──────►│
```

**Vantaggi:**
- ✅ Zero input lag percepito
- ✅ Server autoritativo (anti-cheat)
- ✅ Supporto per alta latenza

### Tech Stack

**Game Engine:**
- **Rust** - Performance e sicurezza della memoria
- **Bevy 0.14** - ECS game engine moderno
- **bevy_renet** - Networking UDP ottimizzato
- **bincode** - Serializzazione efficiente

**Blockchain (Futuro):**
- **Solidity** - Smart contracts
- **Gnosis Chain** - xDAI per basse fee
- **Hardhat** - Testing e deployment

**Frontend (Futuro):**
- **React + TypeScript** - UI moderna
- **WASM** - Game engine nel browser
- **ethers.js** - Interazione blockchain

## 📚 Documentazione

- 📖 [Readme.txt](Readme.txt) - Roadmap dettagliata e progresso
- 🎮 [Game Design](docs/game-design.md) *(coming soon)*
- 🔧 [Architecture](docs/architecture.md) *(coming soon)*
- 💰 [Tokenomics](docs/tokenomics.md) *(coming soon)*

## 🤝 Contributing

Questo progetto è attualmente in sviluppo attivo. Contributi, suggerimenti e feedback sono benvenuti!

```bash
# Fork il repository
# Crea un branch per la tua feature
git checkout -b feature/amazing-feature

# Commit delle modifiche
git commit -m 'Add amazing feature'

# Push del branch
git push origin feature/amazing-feature

# Apri una Pull Request
```

## 📝 License

Questo progetto è rilasciato sotto licenza MIT. Vedi [LICENSE](LICENSE) per dettagli.

## 🎯 Roadmap

### Q1 2025
- ✅ Networking base
- ✅ Player movement & prediction
- 🚧 FPS completo con shooting
- 🚧 Ambiente voxel

### Q2 2025
- 📋 Build WASM del client
- 📋 React frontend con UI per modalità
- 📋 Sistema di lobby (1v1 to 4v4)
- 📋 Smart contracts su testnet (bet mode)

### Q3 2025
- 📋 Matchmaking system per practice mode
- 📋 Ranked matchmaking per bet mode
- 📋 Team balancing algorithms
- 📋 Deployment su Gnosis mainnet

### Q4 2025
- 📋 Mobile client
- 📋 Seasonal tournaments
- 📋 Leaderboards globali
- 📋 DAO governance per prize pools

## 🔗 Links

- 🌐 **Website:** *(coming soon)*
- 🐦 **Twitter:** *(coming soon)*
- 💬 **Discord:** *(coming soon)*
- 📺 **YouTube:** *(coming soon)*

---

**⚠️ Disclaimer:** Questo progetto è in fase di sviluppo attivo. Le feature possono cambiare e il codice potrebbe contenere bug.

**Ultimo aggiornamento:** 18 Gennaio 2025