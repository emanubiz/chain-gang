# Chain Gang — Roadmap

---

## ✅ Completed Fixes

- Mouse sensitivity lowered (0.003 → 0.0018)
- Remote player position lerp (reduces stuttering)
- Local player correction threshold (reduces micro-jitter from reconciliation)
- Remote player hidden on death (`Visibility::Hidden`), reappears on respawn
- Damage numbers visible via centered HUD (old `Text2dBundle` didn't work without `Camera2d`)
- Hit animation: 3D sparks at impact point + damage number in HUD that fades

---

## 🔜 Next Priority: Lobby / Room System

### A. Web Portal — "Rooms" Page
- List active rooms: name, connected players / max, status (waiting / in game)
- "Create Room" and "Join" buttons
- Each room: unique ID, name, max players (e.g. 4v4), map
- Communication with game server via REST API or WebSocket
- On "Join" click: launch client with server_addr + room_id as parameters

### B. Client — Connection Screen (Bevy)
- Initial screen instead of connecting directly on startup:
  - Server IP/port input
  - Username input
  - "Connect" / "Find Matches" button
- Implementation: `AppState::Menu` → `AppState::InGame`
- Menu can also show room list (if server exposes an endpoint)

### C. Game Server — Room Management
- Structure `Room { id, name, players: Vec<ClientId>, max_players, game_state }`
- REST Endpoints:
  - `GET  /rooms`           — list rooms
  - `POST /rooms`           — create room
  - `POST /rooms/{id}/join` — join room
- When a room is full → start match automatically
- When a player dies: becomes spectator until round reset
- Round reset: all players respawned simultaneously

---

## 🔮 Suggested Future Features (in order)

| # | Feature | Notes |
|---|---------|-------|
| 1 | **Scoreboard** | In-game K/D table, press TAB |
| 2 | **Round system** | N kills to win room, then reset |
| 3 | **Spectator Mode** | After death follow another player |
| 4 | **Sounds** | Shot, hit, footsteps — `bevy_kira_audio` |
| 5 | **Animations** | FPS weapon bob, walking legs (remote) |
| 6 | **Multiple Maps** | Map selection from lobby |
| 7 | **Persistent Statistics** | K/D, win/loss on DB (PostgreSQL / SQLite) |
