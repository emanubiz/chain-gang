# Chain Gang — Roadmap

> Aggiornata: 2026-03-22

---

## ✅ Fix completati (sessione 2026-03-22)

- Sensibilità mouse abbassata (0.003 → 0.0018)
- Lerp posizioni giocatori remoti (riduce stuttering)
- Threshold correzione giocatore locale (riduce micro-jitter da reconciliation)
- Giocatore remoto nascosto su morte (`Visibility::Hidden`), riappare al respawn
- Damage numbers visibili via HUD centrato (il vecchio `Text2dBundle` non funzionava senza `Camera2d`)
- Hit animation: scintille 3D al punto di impatto + numero danno nel HUD che sfuma

---

## 🔜 Prossima priorità: Lobby / Room System

### A. Web Portal — Pagina "Rooms"
- Lista room attive: nome, giocatori connessi / max, stato (in attesa / in gioco)
- Pulsanti "Crea Room" e "Unisciti"
- Ogni room: ID univoco, nome, max players (es. 4v4), mappa
- Comunicazione con game server via API REST o WebSocket
- Al click "Unisciti": lancia il client con server_addr + room_id come parametro

### B. Client — Schermata di connessione (Bevy)
- Prima schermata invece di connettersi direttamente all'avvio:
  - Input IP/porta server
  - Input username
  - Pulsante "Connetti" / "Cerca partite"
- Implementazione: `AppState::Menu` → `AppState::InGame`
- Il menu può mostrare anche la lista room (se il server espone un endpoint)

### C. Game Server — Room Management
- Struttura `Room { id, name, players: Vec<ClientId>, max_players, game_state }`
- Endpoint REST:
  - `GET  /rooms`           — lista room
  - `POST /rooms`           — crea room
  - `POST /rooms/{id}/join` — entra in room
- Quando una room è piena → inizia il match automaticamente
- Quando un giocatore muore: diventa spettatore fino al reset del round
- Reset round: tutti rispawnati simultaneamente

---

## 🔮 Feature future (ordine suggerito)

| # | Feature | Note |
|---|---------|------|
| 1 | **Scoreboard** | Tabella K/D in-game, premi TAB |
| 2 | **Round system** | N kill per vincere la room, poi reset |
| 3 | **Modalità spettatore** | Dopo la morte segui un altro giocatore |
| 4 | **Suoni** | Sparo, hit, passi — `bevy_kira_audio` |
| 5 | **Animazioni** | Bob arma in FPS, gambe che camminano (remoto) |
| 6 | **Mappe multiple** | Selezione mappa dalla lobby |
| 7 | **Statistiche persistenti** | K/D, win/loss su DB (PostgreSQL / SQLite) |
