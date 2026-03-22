// game_shared/src/character_constants.rs

/// Costanti di gameplay
pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_HEIGHT: f32 = 1.8;

// ── COSTANTI VOXEL PERSONAGGIO ───────────────────────────────────────
// Il parent entity è posizionato a Y=0 quando i piedi toccano terra.
// Tutti gli offset sono relativi a questa origine (piedi = Y=0).
pub const VOXEL_SCALE: f32 = 0.18;

// TESTA  (center absolute: 1.72, top ~1.94)
pub const HEAD_SIZE: f32 = 0.44;
pub const HEAD_Y_OFFSET: f32 = 1.72;

// CORPO  (center absolute: 1.16, bottom 0.82, top 1.50)
pub const BODY_WIDTH: f32 = 0.40;
pub const BODY_DEPTH: f32 = 0.28;
pub const BODY_HEIGHT: f32 = 0.68;
pub const BODY_Y_OFFSET: f32 = 1.16;

// BRACCIA — offset relativo al centro del CORPO
// center assoluto = BODY_Y_OFFSET + ARM_Y_OFFSET = 1.16
pub const ARM_WIDTH: f32 = 0.16;
pub const ARM_DEPTH: f32 = 0.16;
pub const ARM_HEIGHT: f32 = 0.62;
pub const ARM_Y_OFFSET: f32 = 0.0;

// GAMBE — offset relativo al centro del CORPO
// center assoluto = BODY_Y_OFFSET + LEG_Y_OFFSET = 1.16 - 0.68 = 0.48
// bottom = 0.48 - 0.34 = 0.14  (sopra la suola scarpe)
pub const LEG_WIDTH: f32 = 0.18;
pub const LEG_DEPTH: f32 = 0.18;
pub const LEG_HEIGHT: f32 = 0.68;
pub const LEG_Y_OFFSET: f32 = -0.68;

// SCARPE — offset relativo al centro del CORPO
// shoe_height = 0.10, shoe_platform = 0.04
// center assoluto = 0.09  → relativo a body = 0.09 - 1.16 = -1.07
// piattaforma bottom = 0.00  ✓
pub const SHOE_Y_OFFSET: f32 = -1.07;