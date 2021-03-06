use cardano::{address::{ExtendedAddr, StakeDistribution}, util::{base58, hex, try_from_slice::{TryFromSlice}}, hash};
use cardano_storage::utxo;
use utils::term::{emoji, Term, style::Style};
use std::{io::{self, Read}, str::FromStr};
use exe_common::parse_genesis_data;

pub fn command_address( mut term: Term
                      , address: String
                      )
{
    let bytes = match base58::decode(&address) {
        Err(err) => {
            term.error(&format!("Invalid Address, should be encoded in base58\n")).unwrap();
            term.error(&format!("{}\n", err)).unwrap();
            ::std::process::exit(1)
        },
        Ok(bytes) => bytes,
    };

    let address = match ExtendedAddr::try_from_slice(&bytes) {
        Err(err) => {
            term.error(&format!("Invalid Address\n")).unwrap();
            term.error(&format!("{:?}\n", err)).unwrap();
            ::std::process::exit(2)
        },
        Ok(address) => address,
    };

    term.success("Cardano Extended Address\n").unwrap();
    term.info(&format!("  - address hash:       {}\n", address.addr)).unwrap();
    term.info(&format!("  - address type:       {}\n", address.addr_type)).unwrap();
    if let Some(ref payload) = address.attributes.derivation_path {
        term.info(&format!("  - payload:            {}\n", hex::encode(payload.as_ref()))).unwrap();
    }
    match address.attributes.stake_distribution {
        StakeDistribution::BootstrapEraDistr =>
           term.info("  - stake distribution: bootstrap era\n").unwrap(),
        StakeDistribution::SingleKeyDistr(id) =>
           term.info(&format!("  - stake distribution: {}\n", id)).unwrap(),
    }
}

/// Read a JSON file from stdin and write its canonicalized form to stdout.
pub fn canonicalize_json()
{
    let mut json = String::new();
    io::stdin().read_to_string(&mut json).expect("Cannot read stdin.");
    print!("{}", parse_genesis_data::canonicalize_json(json.as_bytes()));
}

/// Compute the Blake2b256 hash of the data on stdin.
pub fn hash()
{
    let mut data = vec![];
    io::stdin().read_to_end(&mut data).expect("Cannot read stdin.");
    println!("{}", hash::Blake2b256::new(&data));
}

pub fn decode_utxos() {
    let mut data = vec![];
    io::stdin().read_to_end(&mut data).expect("Cannot read stdin.");
    println!("{:?}", utxo::decode_utxo_file(&mut &data[..]).unwrap());
}

pub fn decode_signed_tx() {
    let mut data = String::new();
    io::stdin().read_to_string(&mut data).expect("Cannot read stdin.");

    let bytes = base64::decode(&data).unwrap();
    let txaux : cardano::tx::TxAux = cbor_event::de::RawCbor::from(&bytes).deserialize_complete().unwrap();

    println!("inputs({})", txaux.tx.inputs.len());
    for ((i, input), witness) in txaux.tx.inputs.iter().enumerate().zip(txaux.witness.iter()) {
        let signature_ok = witness.verify_tx(Default::default(), &txaux.tx);
        let valid = if signature_ok {
            emoji::CHECK_MARK
        } else {
            emoji::CHECK_MARK
        };
        println!(" - input ({}) {}.{} {}", i, style!(&input.id), style!(&input.index), valid);
    }

    println!("outputs({}):", txaux.tx.outputs.len());
    for (i, output) in txaux.tx.outputs.iter().enumerate() {
        println!(" - output ({}) {} {}", i, style!(&output.address), style!(&output.value));
    }
}
