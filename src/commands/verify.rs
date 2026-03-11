use crate::error::Result;
use crate::identity;
use crate::storage::{db, Workspace};

pub struct VerifyArgs {
    pub id: String,
    pub json: bool,
}

pub fn run(args: &VerifyArgs) -> Result<()> {
    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;
    let artifact = db::get_artifact_full(&conn, &args.id)?;

    let (signature, signer_pubkey) = match (&artifact.signature, &artifact.signer_pubkey) {
        (Some(sig), Some(pk)) => (sig.clone(), pk.clone()),
        _ => {
            if args.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "id": artifact.id,
                        "signed": false,
                        "status": "unsigned/legacy"
                    })
                );
            } else {
                println!("Artifact  {}", artifact.id);
                println!("Status    unsigned/legacy — no signature to verify");
            }
            return Ok(());
        }
    };

    let payload = identity::signing::build_signing_payload(
        &artifact.content_hash,
        artifact.kind.as_deref(),
        &artifact.created_at,
        artifact.agent_id.as_deref(),
    );

    let valid = identity::signing::verify(&signer_pubkey, &payload, &signature)?;

    let signer_label = identity::keys::load_identity_meta(
        &ws.identities_dir(),
        artifact.signer_address.as_deref().unwrap_or(""),
    )
    .ok()
    .and_then(|id| id.label);

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "id": artifact.id,
                "signed": true,
                "verified": valid,
                "signer_address": artifact.signer_address,
                "signer_label": signer_label,
            })
        );
    } else {
        println!("Artifact  {}", artifact.id);
        println!(
            "Signer    {}{}",
            artifact.signer_address.as_deref().unwrap_or("unknown"),
            signer_label
                .as_deref()
                .map(|l| format!(" ({})", l))
                .unwrap_or_default()
        );
        if valid {
            println!("Status    verified ✓");
        } else {
            println!("Status    FAILED — signature does not match");
            std::process::exit(1);
        }
    }

    if !valid {
        std::process::exit(1);
    }

    Ok(())
}
