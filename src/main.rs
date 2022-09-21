use std::time::Duration;

use libp2p::{
    core::transport::MemoryTransport,
    core::{identity::PublicKey, transport::upgrade, upgrade::SelectUpgrade},
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    identity, mplex,
    multiaddr::Protocol,
    noise,
    swarm::SwarmBuilder,
    yamux, Multiaddr, Swarm, Transport, NetworkBehaviour,
};

fn main() {}

#[tokio::test]
#[serial_test::serial]
async fn test1() {
    println!("test1 start");
    build_swarm(create_local_bind_addr_from_port(1337));
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("test1 end");
}

#[tokio::test]
#[serial_test::serial]
async fn test2() {
    println!("test2 start");
    build_swarm(create_local_bind_addr_from_port(1337));
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("test2 end");
}

fn create_local_bind_addr_from_port(port: u16) -> Multiaddr {
    let mut addr = Multiaddr::empty();
    addr.push(Protocol::Memory(port as u64));
    addr
}

fn build_swarm(bind_addr: Multiaddr) {
    let local_keys = identity::Keypair::generate_ed25519();
    let local_public_key = local_keys.public();
    let local_id = local_public_key.to_peer_id();

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_keys)
        .unwrap();

    let noise_config = noise::NoiseConfig::xx(noise_keys);
    let mplex_config = mplex::MplexConfig::default();
    let yamux_config = yamux::YamuxConfig::default();

    let transport = MemoryTransport::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_config.into_authenticated())
        .multiplex(SelectUpgrade::new(yamux_config, mplex_config))
        .timeout(Duration::from_secs(10))
        .boxed();

    let behaviour = SwarmBehaviour::new(local_public_key);
    let mut swarm = SwarmBuilder::new(transport, behaviour, local_id).build();

    Swarm::listen_on(&mut swarm, bind_addr).unwrap();
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "SwarmBehaviourEvent")]
pub struct SwarmBehaviour {
    identify: Identify,
}

impl SwarmBehaviour {
    pub fn new(local_public_key: PublicKey) -> Self {
        let config = IdentifyConfig::new("test/0.1.0".to_string(), local_public_key);

        Self {
            identify: Identify::new(config),
        }
    }
}

pub enum SwarmBehaviourEvent {
    Identify(Box<IdentifyEvent>),
}

impl From<IdentifyEvent> for SwarmBehaviourEvent {
    fn from(event: IdentifyEvent) -> Self {
        SwarmBehaviourEvent::Identify(Box::new(event))
    }
}
