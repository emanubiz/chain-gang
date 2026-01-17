use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use game_shared::{hello_shared, PROTOCOL_ID, SERVER_PORT, SERVER_ADDR, PhysicsMessage};
use bevy_renet::renet::{ConnectionConfig, RenetServer, ServerEvent};
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::RenetServerPlugin;
use bevy_rapier3d::prelude::*;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

// Wrapper Resource per NetcodeServerTransport
#[derive(Resource)]
struct Transport(NetcodeServerTransport);

// Risorsa per tracciare l'entità del cubo sincronizzato
#[derive(Resource)]
struct SynchronizedCube(Entity);

fn main() {
    println!("🔥 SERVER: Avvio in corso...");
    hello_shared();

    let mut app = App::new();
    
    app.add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0)
            ))
        )
        .add_plugins(RenetServerPlugin);
    
    // Aggiungiamo Rapier manualmente con TUTTI gli eventi e risorse necessarie
    app.init_resource::<RapierContext>()
        .init_resource::<Events<CollisionEvent>>()
        .init_resource::<Events<ContactForceEvent>>()
        .init_resource::<Events<MassModifiedEvent>>()
        .init_resource::<SimulationToRenderTime>()
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81,
            physics_pipeline_active: true,
            query_pipeline_active: true,
            timestep_mode: TimestepMode::Variable {
                max_dt: 1.0 / 60.0,
                time_scale: 1.0,
                substeps: 1,
            },
            scaled_shape_subdivision: 10,
            force_update_from_transform_changes: false,
        });
    
    // Sistemi in First (prima di tutto)
    app.add_systems(
        First,
        bevy_rapier3d::plugin::systems::sync_removals,
    );
    
    // Sistemi in PreUpdate (prima dell'update principale)
    app.add_systems(
        PreUpdate,
        (
            bevy_rapier3d::plugin::systems::apply_scale,
            bevy_rapier3d::plugin::systems::apply_collider_user_changes,
            bevy_rapier3d::plugin::systems::apply_rigid_body_user_changes,
            bevy_rapier3d::plugin::systems::apply_joint_user_changes,
            bevy_rapier3d::plugin::systems::apply_initial_rigid_body_impulses,
        ).chain(),
    );
    
    // Step della simulazione in Update
    app.add_systems(
        Update,
        bevy_rapier3d::plugin::systems::step_simulation::<NoUserData>,
    );
    
    // Writeback in PostUpdate (dopo l'update)
    app.add_systems(
        PostUpdate,
        (
            bevy_rapier3d::plugin::systems::update_colliding_entities,
            bevy_rapier3d::plugin::systems::writeback_rigid_bodies,
        ),
    );
    
    app.add_systems(Startup, (setup_network, setup_physics).chain())
        .add_systems(Update, (
            update_transport,
            handle_server_events,
            server_tick,
        ))
        .add_systems(PostUpdate, sync_physics_to_clients)
        .run();
}

fn setup_network(mut commands: Commands) {
    let server_addr = format!("{}:{}", SERVER_ADDR, SERVER_PORT).parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // Crea socket UDP
    let socket = UdpSocket::bind(server_addr)
        .expect("Impossibile bindare il socket UDP del server");

    // Crea RenetServer
    let server = RenetServer::new(ConnectionConfig::default());

    // Configurazione transport
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    // Crea transport
    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Impossibile creare NetcodeServerTransport");

    commands.insert_resource(server);
    commands.insert_resource(Transport(transport));
}

fn setup_physics(
    mut commands: Commands,
) {
    println!("🔧 SERVER: Setup fisica in corso...");
    
    // Spawna il pavimento (collider statico)
    let floor = commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.5, 10.0),
        TransformBundle::from(Transform::from_xyz(0.0, -0.5, 0.0)),
    )).id();
    println!("✅ SERVER: Pavimento creato: {:?}", floor);

    // Spawna il cubo che cade (corpo dinamico)
    let cube_entity = commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        TransformBundle::from(Transform::from_xyz(0.0, 5.0, 0.0)),
        Restitution::coefficient(0.7),
        Velocity::default(), // Aggiungiamo velocità esplicita
        GravityScale(1.0),   // Aggiungiamo scala di gravità
    )).id();

    // Salva l'entità del cubo per sincronizzarla
    commands.insert_resource(SynchronizedCube(cube_entity));
    
    println!("✅ SERVER: Cubo fisico spawnato con ID: {:?}", cube_entity);
}

fn update_transport(
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    
    // Aggiorna il transport
    if let Err(e) = transport.0.update(delta, &mut *server) {
        eprintln!("❌ Errore transport: {:?}", e);
    }
}

fn sync_physics_to_clients(
    mut server: ResMut<RenetServer>,
    synced_cube: Res<SynchronizedCube>,
    query: Query<(&Transform, Option<&Velocity>)>,
    context: Res<RapierContext>,
    time: Res<Time>,
) {
    // Debug: stampa ogni 2 secondi info sulla fisica
    if (time.elapsed_seconds() / 2.0).floor() != ((time.elapsed_seconds() - time.delta_seconds()) / 2.0).floor() {
        let client_count = server.clients_id().len();
        if client_count > 0 {
            println!("🔄 Sincronizzando fisica con {} client(i)", client_count);
        }
        
        // Debug: verifica se il RapierContext ha entità
        println!("🔍 DEBUG: Entità nel RapierContext: {}", context.entity2body().len());
    }

    // Ottieni la transform del cubo
    if let Ok((transform, velocity)) = query.get(synced_cube.0) {
        let message = PhysicsMessage::RigidBodyUpdate {
            entity_id: synced_cube.0.index() as u64,
            position: transform.translation,
            rotation: transform.rotation,
        };

        // Debug: stampa la posizione e velocità del cubo
        if (time.elapsed_seconds() / 2.0).floor() != ((time.elapsed_seconds() - time.delta_seconds()) / 2.0).floor() {
            if let Some(vel) = velocity {
                println!("📦 Cubo a posizione: {:?}, velocità: {:?}", transform.translation, vel.linvel);
            } else {
                println!("📦 Cubo a posizione: {:?}, NO VELOCITY COMPONENT", transform.translation);
            }
        }

        // Serializza e invia a tutti i client connessi
        if let Ok(message_data) = bincode::serialize(&message) {
            for client_id in server.clients_id() {
                server.send_message(client_id, 0, message_data.clone());
            }
        }
    }
}

fn handle_server_events(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("✅ SERVER: Client {} si è connesso.", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("❌ SERVER: Client {} si è disconnesso: {:?}", client_id, reason);
            }
        }
    }
}

fn server_tick(time: Res<Time>) {
    let elapsed = time.elapsed_seconds();
    let prev_elapsed = elapsed - time.delta_seconds();
    
    if (elapsed / 2.0).floor() != (prev_elapsed / 2.0).floor() {
        println!("🔄 SERVER: Tick: {:.2}s", elapsed);
    }
}