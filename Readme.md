# 🎮 CHAIN GANG

> **Tactical FPS meets DeFi** - A competitive multiplayer game where every match has real value.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.14-blue.svg)](https://bevyengine.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## 🚀 What is Chain Gang?

Chain Gang is a **tactical first-person shooter** that combines skill-based competitive gameplay with blockchain.

### 🎯 Game Modes

**📊 Flexibility:** From **1v1** to **4v4**
- 🥊 **1v1** - Intense one-on-one duels
- 👥 **2v2** - Pair cooperation
- 🎮 **3v3** - Classic team tactics
- 🏆 **4v4** - Complete team battles

**🎲 Two Modes:**

1. **⚡ Practice Mode** (Free Play)
   - Free play without betting
   - Perfect for training and fun
   - Casual matchmaking
   - Zero risk, pure skill

2. **💰 Betting Mode** (Ranked + DeFi)
   - Competitive matches with stake
   - Teams lock xDAI on smart contracts
   - Winners receive the full prize pool immediately
   - **Zero intermediaries, 100% trustless**

### ✨ Main Features

- 🎯 **Competitive FPS** - Skill-based gameplay with bullet physics and friendly fire
- 👥 **1v1 up to 4v4** - Flexible game modes for every style
- 🎲 **Dual Mode** - Free practice + On-chain betting
- 🌐 **Cross-Platform** - Web (WASM) and Mobile
- 💰 **Integrated DeFi** - Smart contracts on Gnosis Chain for trustless escrow
- 🎨 **Voxel Aesthetic** - Modern low-poly style
- ⚡ **Client-Side Prediction** - Zero perceived input lag
- 🔒 **Authoritative Server** - Native anti-cheat in architecture

## 📦 Project Structure

```
chain-gang/
├── 🎮 game-engine/        # Game core (Rust + Bevy)
│   ├── game_shared/       # Shared client-server logic
│   ├── game_server/       # Authoritative server
│   └── game_client/       # Game client
├── ⚛️  web-portal/        # React + WASM frontend
├── ⛓️  contracts/         # Solidity smart contracts
└── 🐳 infrastructure/    # Docker & deployment
```

## 🎯 Project Status

**Current Phase:** PHASE 1 - Game Core

| Step | Description | Status |
|------|-------------|--------|
| 1.1 | Networking Skeleton | ✅ Completed |
| 1.2 | Synchronized Physics | ✅ Completed |
| 1.3 | Player Movement & Prediction | ✅ Completed |
| 1.4 | Player Experience Refinement | 🚧 In Progress |
| 1.5 | Voxel & Shooting | 📋 Planned |
| 1.6 | Game Modes & Lobby | 📋 Planned |

**Step 1.4 - Player Experience Refinement:**
- 🎯 Mouse look (camera rotation)
- 🎯 FPS camera following the player
- 🎯 Remote player interpolation (smooth movement)
- 🎯 Complete reconciliation (reapply pending inputs)

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Execution

```bash
# Clone the repository
git clone https://github.com/yourusername/chain-gang.git
cd chain-gang/game-engine

# Terminal 1: Start the server
cargo run --bin game_server

# Terminal 2: Start the client
cargo run --bin game_client
```

### 🎮 Controls

| Key | Action |
|-----|--------|
| **W** | Forward |
| **A** | Left |
| **S** | Backward |
| **D** | Right |
| **Space** | Jump |

## 🎮 How It Works

### Practice Mode (Free Play)

```
1. 🔍 Enter matchmaking
2. 🎯 Choose mode (1v1, 2v2, 3v3, 4v4)
3. ⚔️  Play and have fun
4. 📈 Improve your skills
```

**Zero risk, just fun!**

### Betting Mode (Ranked)

```
1. 💰 Teams lock xDAI on smart contract
   └─ Example: 4v4 → 10 xDAI per team = 20 xDAI pot

2. 🎮 Match starts (authoritative server)
   └─ Skill-based gameplay with anti-cheat

3. 🏆 Winning team receives cryptographic proof
   └─ Verifiable server signature on-chain

4. ⚡ Immediate prize unlock (20 xDAI)
   └─ Trustless, no intermediaries
   └─ Automatically distributed to team members
```

**100% Trustless. 100% Skill-Based.**

### Networking

Chain Gang uses an **authoritative client-server architecture** with **client-side prediction**:

```
Client                          Server
   │                               │
   ├─► PlayerInput (seq: 1) ──────►│
   │   (predicts locally)          │ Processes input
   │                               │ Simulates physics
   │◄── PlayerState (seq: 1) ──────┤ 
   │   (reconciles if different)   │
   └─► PlayerInput (seq: 2) ──────►│
```

**Advantages:**
- ✅ Zero perceived input lag
- ✅ Authoritative server (anti-cheat)
- ✅ High latency support

### Tech Stack

**Game Engine:**
- **Rust** - Performance and memory safety
- **Bevy 0.14** - Modern ECS game engine
- **bevy_renet** - Optimized UDP networking
- **bincode** - Efficient serialization

**Blockchain (Future):**
- **Solidity** - Smart contracts
- **Gnosis Chain** - xDAI for low fees
- **Hardhat** - Testing and deployment

**Frontend (Future):**
- **React + TypeScript** - Modern UI
- **WASM** - Game engine in browser
- **ethers.js** - Blockchain interaction

## 📚 Documentation

- 📖 [Readme.txt](Readme.txt) - Detailed roadmap and progress
- 🎮 [Game Design](docs/game-design.md) *(coming soon)*
- 🔧 [Architecture](docs/architecture.md) *(coming soon)*
- 💰 [Tokenomics](docs/tokenomics.md) *(coming soon)*

## 🤝 Contributing

This project is actively developed. Contributions, suggestions and feedback are welcome!

```bash
# Fork the repository
# Create a branch for your feature
git checkout -b feature/amazing-feature

# Commit your changes
git commit -m 'Add amazing feature'

# Push the branch
git push origin feature/amazing-feature

# Open a Pull Request
```

## 📝 License

This project is released under the MIT license. See [LICENSE](LICENSE) for details.

## 🎯 Roadmap

### 
- ✅ Networking base
- ✅ Player movement & prediction
- 🚧 Complete FPS with shooting
- 🚧 Voxel environment

### 
- 📋 Build WASM client
- 📋 React frontend with mode UI
- 📋 Lobby system (1v1 to 4v4)
- 📋 Smart contracts on testnet (bet mode)

### 
- 📋 Practice mode matchmaking system
- 📋 Ranked matchmaking for bet mode
- 📋 Team balancing algorithms
- 📋 Deployment to Gnosis mainnet

### 
- 📋 Mobile client
- 📋 Seasonal tournaments
- 📋 Global leaderboards
- 📋 DAO governance for prize pools

## 🔗 Links

- 🌐 **Website:** *(coming soon)*
- 🐦 **Twitter:** *(coming soon)*
- 💬 **Discord:** *(coming soon)*
- 📺 **YouTube:** *(coming soon)*

---

**⚠️ Disclaimer:** This project is in active development. Features may change and the code may contain bugs.

**Last Updated:** January 18, 2025