use ark_ff::{BigInteger, PrimeField};
use co_noir::{
    Address, Bn254, CrsParser, NetworkConfig, NetworkParty, PartyID, Poseidon2Sponge,
    Rep3CoUltraHonk, Rep3MpcNet, UltraHonk, Utils,
};
use co_ultrahonk::prelude::ZeroKnowledge;
use color_eyre::{Result, eyre::Context};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::path::PathBuf;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    prelude::*,
};

fn main() -> Result<()> {
    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::CLOSE | FmtSpan::ENTER);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    let dir = PathBuf::from("data");

    // connect to network
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(std::fs::read(
        dir.join("key0.der"),
    )?))
    .clone_key();
    let parties = vec![
        NetworkParty::new(
            PartyID::ID0.into(),
            Address::new("localhost".to_string(), 10000),
            CertificateDer::from(std::fs::read(dir.join("cert0.der"))?).into_owned(),
        ),
        NetworkParty::new(
            PartyID::ID1.into(),
            Address::new("localhost".to_string(), 10001),
            CertificateDer::from(std::fs::read(dir.join("cert1.der"))?).into_owned(),
        ),
        NetworkParty::new(
            PartyID::ID2.into(),
            Address::new("localhost".to_string(), 10002),
            CertificateDer::from(std::fs::read(dir.join("cert2.der"))?).into_owned(),
        ),
    ];
    let network_config = NetworkConfig::new(0, "0.0.0.0:10000".parse()?, key, parties, None);
    let mut net = Rep3MpcNet::new(network_config)?;

    println!("✅ MPC network initialized. Result = {:?}", net);
    let dir = PathBuf::from("../noir");

    // parse constraint system
    let program_artifact = Utils::get_program_artifact_from_file(dir.join("target/noir.json"))
        .context("while parsing program artifact")?;
    let constraint_system = Utils::get_constraint_system_from_artifact(&program_artifact, true);

    // read the input file
    let inputs = co_noir::parse_input(dir.join("Prover.toml"), &program_artifact)?;

    let recursive = true;
    let has_zk = ZeroKnowledge::No;

    // parse crs
    let crs_size = co_noir::compute_circuit_size::<Bn254>(&constraint_system, recursive)?;
    let (prover_crs, verifier_crs) = CrsParser::<Bn254>::get_crs(
        dir.join("bn254/bn254_g1.dat"),
        dir.join("bn254/bn254_g2.dat"),
        crs_size,
        has_zk,
    )?
    .split();

    // create input shares
    let mut rng = rand::thread_rng();
    let [share0, share1, share2] =
        co_noir::split_input_rep3::<Bn254, Rep3MpcNet, _>(inputs, &mut rng);

    // send shares to other parties
    net.send_bytes(PartyID::ID1, bincode::serialize(&share1)?.into())?;
    net.send_bytes(PartyID::ID2, bincode::serialize(&share2)?.into())?;

    // generate witness
    let (witness_share, net) = co_noir::generate_witness_rep3(share0, program_artifact, net)?;

    // generate proving key and vk
    let (pk, net) =
        co_noir::generate_proving_key_rep3(net, &constraint_system, witness_share, recursive)?;
    let vk = pk.create_vk(&prover_crs, verifier_crs)?;
    let vk_save = vk.clone().to_barrettenberg().to_buffer();
    std::fs::write("verification_key", &vk_save)?;

    // generate proof
    let (proof, public_inputs, _) =
        Rep3CoUltraHonk::<_, _, Poseidon2Sponge>::prove(net, pk, &prover_crs, has_zk)?;
    std::fs::write("proof", proof.to_buffer())?;

    // Konwertuj public inputs na bajty
    let mut public_inputs_bytes = Vec::new();
    for el in &public_inputs {
        let bytes = el.into_bigint().to_bytes_be();
        public_inputs_bytes.extend_from_slice(&bytes);
    }
    std::fs::write("public_inputs", public_inputs_bytes)?;

    // verify proof
    assert!(
        UltraHonk::<_, Poseidon2Sponge>::verify(proof, &public_inputs, &vk, has_zk)
            .context("while verifying proof")?
    );
    Ok(())
}
