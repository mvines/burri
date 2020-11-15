use {
    clap::{crate_description, crate_name, crate_version, Arg, Command},
    rand::Rng,
    solana_clap_v3_utils::{
        input_parsers::pubkeys_of,
        input_validators::{
            is_url_or_moniker, is_valid_pubkey, is_valid_signer, normalize_to_url_if_moniker,
        },
        keypair::DefaultSigner,
    },
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, Instruction},
        message::Message,
        native_token::Sol,
        pubkey::Pubkey,
        system_instruction, system_program,
        transaction::Transaction,
    },
    std::{process::exit, sync::Arc},
};

pub fn transfer_with(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
    extra_addresses: &[Pubkey],
) -> Instruction {
    let mut account_metas = vec![
        AccountMeta::new(*from_pubkey, true),
        AccountMeta::new(*to_pubkey, false),
    ];
    for extra_address in extra_addresses {
        account_metas.push(AccountMeta::new_readonly(*extra_address, false));
    }

    Instruction::new_with_bincode(
        system_program::id(),
        &system_instruction::SystemInstruction::Transfer { lamports },
        account_metas,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg_required_else_help(true)
        .arg({
            let arg = Arg::new("config_file")
                .short('C')
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::new("keypair")
                .long("keypair")
                .value_name("KEYPAIR")
                .validator(|s| is_valid_signer(s))
                .takes_value(true)
                .help("Filepath or URL to a keypair [default: client keypair]"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .takes_value(false)
                .help("Show additional information"),
        )
        .arg(
            Arg::new("json_rpc_url")
                .short('u')
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(|s| is_url_or_moniker(s))
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .arg(
            Arg::new("extra_addresses")
                .value_name("ADDRESS")
                .validator(|s| is_valid_pubkey(s))
                .takes_value(true)
                .multiple(true)
                .help("Extra addresses to append"),
        )
        .get_matches();

    let mut wallet_manager: Option<Arc<RemoteWalletManager>> = None;

    let cli_config = if let Some(config_file) = matches.value_of("config_file") {
        solana_cli_config::Config::load(config_file).unwrap_or_default()
    } else {
        solana_cli_config::Config::default()
    };

    let default_signer = DefaultSigner::new(
        "keypair",
        matches
            .value_of("keypair")
            .map(|s| s.to_string())
            .unwrap_or_else(|| cli_config.keypair_path.clone()),
    )
    .signer_from_path(&matches, &mut wallet_manager)
    .unwrap_or_else(|err| {
        eprintln!("error: {err}");
        exit(1);
    });

    let json_rpc_url = normalize_to_url_if_moniker(
        matches
            .value_of("json_rpc_url")
            .unwrap_or(&cli_config.json_rpc_url),
    );

    let verbose = matches.is_present("verbose");
    let extra_addresses = pubkeys_of(&matches, "extra_addresses").unwrap_or_default();

    solana_logger::setup_with_default("solana=info");
    if verbose {
        println!("JSON RPC URL: {json_rpc_url}");
    }
    let rpc_client =
        RpcClient::new_with_commitment(json_rpc_url.clone(), CommitmentConfig::confirmed());

    let feepayer_address = default_signer.pubkey();
    let feepayer_balance = rpc_client.get_balance(&default_signer.pubkey()).await?;

    let transfer_amount = rand::thread_rng().gen_range(0..(feepayer_balance / 2));

    if verbose {
        println!(
            "Fee payer: {}, Amount: {}",
            feepayer_address,
            Sol(transfer_amount)
        );
        println!("Extra addresses: {extra_addresses:?}");
    }

    let mut transaction = Transaction::new_unsigned(Message::new(
        &[transfer_with(
            &feepayer_address,
            &feepayer_address,
            transfer_amount,
            &extra_addresses,
        )],
        Some(&feepayer_address),
    ));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {err}"))?;

    transaction
        .try_sign(&vec![default_signer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {err}"))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {err}"))?;

    println!("Signature: {signature}");

    Ok(())
}
