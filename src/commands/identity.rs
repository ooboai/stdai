use crate::cli::IdentityCommands;
use crate::error::Result;
use crate::identity;
use crate::storage::Workspace;

pub fn run(sub: IdentityCommands) -> Result<()> {
    let ws = Workspace::open()?;
    let id_dir = ws.identities_dir();

    match sub {
        IdentityCommands::New { label } => {
            let (signing_key, _) = identity::keys::generate_keypair();
            let id = identity::keys::save_identity(&id_dir, label.as_deref(), &signing_key)?;
            println!("Created identity:\n");
            println!("  address:  {}", id.address);
            if let Some(ref l) = id.label {
                println!("  label:    {}", l);
            }
            println!("  created:  {}", id.created_at);
            println!("  store:    {}", id_dir.display());
            println!();
            println!("Set it for your session:");
            println!();
            println!("  export STDAI_IDENTITY={}", id.address);
        }
        IdentityCommands::List { json } => {
            let identities = identity::keys::list_identities(&id_dir)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&identities)?);
            } else if identities.is_empty() {
                eprintln!("no identities found — create one with: stdai identity new");
            } else {
                println!("{:<50} {:<20} {}", "ADDRESS", "LABEL", "CREATED");
                for id in &identities {
                    println!(
                        "{:<50} {:<20} {}",
                        id.address,
                        id.label.as_deref().unwrap_or("-"),
                        id.created_at,
                    );
                }
            }
        }
        IdentityCommands::Show { address } => {
            let resolved = identity::keys::resolve_address(&id_dir, &address)?;
            let id = identity::keys::load_identity_meta(&id_dir, &resolved)?;
            let vk = identity::keys::load_verifying_key(&id_dir, &resolved)?;
            println!("Address   {}", id.address);
            if let Some(ref l) = id.label {
                println!("Label     {}", l);
            }
            println!("Pubkey    {}", hex::encode(vk.as_bytes()));
            println!("Created   {}", id.created_at);
            println!("Secret    {}", if id.has_secret { "yes (local)" } else { "no (import-only)" });
        }
        IdentityCommands::Export { address } => {
            let resolved = identity::keys::resolve_address(&id_dir, &address)?;
            let vk = identity::keys::load_verifying_key(&id_dir, &resolved)?;
            println!("{}", hex::encode(vk.as_bytes()));
        }
        IdentityCommands::Import { pubkey, label } => {
            let id = identity::keys::import_pubkey(&id_dir, &pubkey, label.as_deref())?;
            println!("Imported identity:\n");
            println!("  address:  {}", id.address);
            if let Some(ref l) = id.label {
                println!("  label:    {}", l);
            }
            println!("  type:     verification-only (no secret key)");
        }
    }

    Ok(())
}
