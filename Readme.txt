# PROJECT: CHAIN GANG

## GLOBAL GAME DESCRIPTION

CHAIN GANG is a tactical first-person shooter (FPS) team-based cross-platform (Web & Mobile) with a Voxel/low-poly aesthetic.

### Game Modes
*   **1v1 up to 4v4** - From 1v1 duels to 4v4 team battles
*   **Practice Mode** - Free play without betting for training and fun
*   **Betting Mode** - Competitive matches with on-chain stake

### DeFi Mechanics (Betting Mode)
Teams lock a stake (in xDAI) in a Gnosis Smart Contract before a match. Gameplay is skill-based, with friendly fire and bullet physics. At the end of the match, the winning team receives a cryptographic proof from the server to immediately and trustlessly unlock the entire prize pool on their wallet, eliminating intermediaries.

## MASTER PLAN (Global Roadmap)

The project will proceed in "playable" phases to gradually build the complete game.

*   **PHASE 1: The Game Core (Rust/Bevy)**
    *   **Objective:** Create a functional multiplayer FPS with movement, shooting and physics.
    *   **Technologies:** Rust, Bevy Engine, bevy_renet (networking)
    *   **Status:** IN PROGRESS - Step 1.3 completed ✅

*   **PHASE 2: Web Integration (WASM + React)**
    *   **Objective:** Run the game in the browser (WASM) and create the React interface around it (Chat, Main Menu).

*   **PHASE 3: Multiplayer Infrastructure (Redis + Matchmaking)**
    *   **Objective:** Manage dynamic rooms and matchmaking to assign players to available rooms.

*   **PHASE 4: DeFi (Smart Contracts)**
    *   **Objective:** Write and test escrow contracts on Gnosis. Integrate wallet login and DeFi interactions.

*   **PHASE 5: The "Glue" (Putting it all together)**
    *   **Objective:** Finalize integration between game server (victory proof), frontend (funds unlock) and blockchain contracts.

## WORKSPACE STRUCTURE (Monorepo)

```
chain-gang/
│
├── 📂 contracts/              (BLOCKCHAIN LAYER)
│   │ # Solidity Smart Contract (Gnosis)
│   ├── contracts/             # .sol files
│   ├── test/                  # Contract tests
│   └── hardhat.config.js
│
├── 📂 web-portal/             (FRONTEND LAYER - React)
│   │ # The website the user visits
│   ├── src/                   # React, Wallet connection, UI
│   ├── public/                # Where the .wasm game will be copied
│   └── package.json
│
├── 📂 game-engine/            (GAME LAYER - Rust Workspace)
│   │ # The heart of the game (Client and Server)
│   ├── Cargo.toml             # Rust Workspace
│   │
│   ├── 📦 game_shared/        # Shared Logic (Protocol, Network Messages)
│   │   └── src/lib.rs         # PlayerInput, NetworkMessage, apply_player_movement()
│   │
│   ├── 📦 game_server/        # The Authoritative Server (Linux Binary)
│   │   └── src/main.rs        # Connection handling, server-side physics, sync
│   │
│   └── 📦 game_client/        # The Visual Game (WASM target)
│       └── src/main.rs        # Client-side prediction, rendering, input
│
└── 📂 infrastructure/         (OPS LAYER)
    └── docker-compose.yml
```

## LOCAL ROADMAP (Phase 1: Game Engine)

### ✅ Step 1.1 - Networking Skeleton
*   Rust workspace setup (3 crates: shared, server, client)
*   Server listening on port 5000
*   Client connecting
*   Connection/disconnection event exchange via `bevy_renet`
*   **STATUS: COMPLETED** ✅

### ✅ Step 1.2 - Synchronized Physics (The Cube)
*   Manual physics integration (no bevy_rapier for simplicity)
*   Server spawns a cube that falls and bounces
*   Server sends cube position/rotation to clients via `NetworkMessage::RigidBodyUpdate`
*   Client displays the synchronized cube
*   Fix of `transport.send_packets()` to actually send UDP packets
*   **STATUS: COMPLETED** ✅

### ✅ Step 1.3 - Player Movement (Client-Side Prediction)
*   **Player Input:** WASD to move, Space to jump
*   **Client-Side Prediction:** Movement applied IMMEDIATELY on client for zero perceived lag
*   **Server Authoritative:** Server receives inputs, processes them and sends updated state
*   **Reconciliation:** Client corrects its position when receiving server updates
*   **Shared Function:** `apply_player_movement()` used by both client and server to ensure consistency
*   **Multi-player:** Multiple player spawns (green for local, red for remote)
*   **STATUS: COMPLETED** ✅

### 🚧 Step 1.4 - Player Experience Refinement
*   **Objective:** Improve game feel and fluidity
*       *   **Camera Rotation** (mouse look) - First Person Control
*       *   **Camera Following Player** - FPS View
*       *   **Interpolation** of remote players for smooth movement (no teleport)
*       *   **Reapplication of pending inputs** after reconciliation for perfect prediction
*   **STATUS: IN PROGRESS** 🎯

### 📋 Step 1.5 - Voxel & Shooting
*   Basic voxel environment generation
*   Shooting logic (Raycasting)
*   Synchronized voxel removal between all clients
*   Hit detection on players
*   Health system and respawn
*   **STATUS: PLANNED**

### 📋 Step 1.6 - Game Modes
*   Lobby system for 1v1, 2v2, 3v3, 4v4
*   Matchmaking for practice mode
*   Match timers and win conditions
*   Scoreboard and match statistics
*   **STATUS: PLANNED**

## TECHNOLOGIES USED

### Game Engine
*   **Rust** - Programming language (performance + safety)
*   **Bevy 0.14** - ECS Game Engine (Entity Component System)
*   **bevy_renet 0.0.12** - Networking library (client-server)
*   **bincode** - Binary serialization for network messages
*   **serde** - Serialization/Deserialization

### Frontend (Future)
*   **React + TypeScript** - UI framework
*   **Vite** - Build tool
*   **Ethers.js** - Blockchain interaction

### Blockchain (Future)
*   **Solidity** - Smart contracts
*   **Hardhat** - Development environment
*   **Gnosis Chain** - Network (xDAI)

## HOW TO RUN THE PROJECT

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Running the Game Engine

```bash
# Terminal 1 - Start the Server
cd game-engine
cargo run --bin game_server

# Terminal 2 - Start the Client
cargo run --bin game_client

# (Optional) Terminal 3 - Second client for multiplayer testing
cargo run --bin game_client
```

### Game Controls
*   **W** - Forward
*   **A** - Left
*   **S** - Backward
*   **D** - Right
*   **Space** - Jump

## IMPORTANT TECHNICAL NOTES

### Networking Architecture
*   **Authoritative Client-Server:** The server is the sole source of truth
*   **Client-Side Prediction:** Client predicts movement for immediate responsiveness
*   **Server Reconciliation:** When server sends updated state, client corrects any divergences
*   **Sequence Numbers:** Each input has a sequence number to track which input the server processed

### Physics
*   **Custom Physics:** Manually implemented (no bevy_rapier) for total control and WASM compatibility
*   **Gravity:** -9.81 m/s²
*   **Floor Collision:** Simple Y <= PLAYER_HEIGHT/2 check
*   **Movement:** 5.0 m/s base speed
*   **Jump:** 5.0 m/s vertical force

### Network Messages
*   **PlayerInput:** Client → Server (WASD input + jump + sequence_number)
*   **PlayerStateUpdate:** Server → Client (position, velocity, rotation + sequence_number)
*   **RigidBodyUpdate:** Server → Client (non-player objects like the cube)
*   **PlayerConnected/Disconnected:** Server → All Clients (notifications)

## NEXT STEPS (Immediate)

1. ✅ Resolve compilation errors (`ClientId` vs `u64`)
2. 🎯 Implement mouse look (camera rotation)
3. 🎯 Camera that follows the player
4. 🎯 Interpolation of remote players
5. 🎯 Reapply pending inputs (complete reconciliation)

## PROJECT STATUS

*   **Current Phase:** PHASE 1 - Step 1.3 completed
*   **Next Milestone:** Step 1.4 - Player Experience Refinement
*   **Phase 1 Completion Level:** ~60%

---

**Note:** This project is under active development. Documentation is updated with each completed step.