use std::{collections::HashMap, io};

use anyhow::{Result, bail};
use fs_err::{create_dir_all, remove_file, rename, write};
use hickory_proto::rr::{RData, RecordType};
use instant_acme::{
  Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier,
  NewAccount, NewOrder, OrderStatus, RetryPolicy,
};
use tracing::{debug, log::error, warn};
use vox_shared::{
  config::{
    Config,
    certs::{AcmeChallenge, CertificateStrategy},
  },
  home_dir,
  pretty::{print_field, print_message, print_separator, print_success},
};

use crate::{
  certs::{Certs, load_certs, renewal::certs_need_renewal},
  dns::resolver::HickoryResolver,
};

impl Certs {
  async fn acme_account(config: &Config) -> Result<Account> {
    let acme_file = home_dir().join("acme-accounts.toml");

    let mut accounts = if acme_file.exists() {
      toml::from_str::<HashMap<String, AccountCredentials>>(&fs_err::read_to_string(
        &acme_file,
      )?)?
    } else {
      HashMap::new()
    };

    let directory_url = config.certs.acme.directory_url.clone();

    if let Some(creds) = accounts.remove(&directory_url) {
      let account = Account::builder()?.from_credentials(creds).await?;

      return Ok(account);
    }

    let contact = config.certs.acme.email.as_ref().map(|email| format!("mailto:{email}"));

    let contact_refs = contact.as_deref().into_iter().collect::<Vec<_>>();

    let (account, credentials) = Account::builder()?
      .create(
        &NewAccount {
          contact: &contact_refs,
          terms_of_service_agreed: true,
          only_return_existing: false,
        },
        directory_url.clone(),
        None,
      )
      .await?;
    debug!(%directory_url, "using acme account for directory:");

    accounts.insert(directory_url, credentials);
    fs_err::write(acme_file, toml::to_string_pretty(&accounts)?)?;

    Ok(account)
  }

  pub fn load_certs_with_acme(config: &Config) -> Result<Certs> {
    let Some(domain) = &config.certs.acme.domain else {
      bail!("no domain provided for ACME certificate");
    };

    let certs_path = home_dir().join("certs");
    let cert_path = certs_path.join(format!("{domain}.pem"));
    let key_path = certs_path.join(format!("{domain}.key"));

    if !cert_path.exists() || !key_path.exists() {
      bail!(
        "no fetched ACME certificate found for {domain}; run `cli.exe acme challenge`"
      );
    }

    let certs = load_certs(&cert_path)?;
    if certs_need_renewal(&certs)? {
      warn!(%domain, "ACME certificate needs renewal; run `cli.exe acme challenge`");
    }

    Self::load(&cert_path, &key_path)
  }

  pub async fn fetch_acme_certificate(
    config: &Config,
    resolver: &HickoryResolver,
  ) -> Result<()> {
    if !matches!(config.certs.strategy, CertificateStrategy::Acme) {
      bail!("certificate strategy must be set to `acme` before running the challenge");
    }

    if !matches!(config.certs.acme.challenge, AcmeChallenge::Dns01) {
      bail!("currently only the DNS-01 challenge is supported");
    }

    let Some(domain) = &config.certs.acme.domain else {
      bail!("no domain provided for ACME challenge");
    };

    let certs_path = home_dir().join("certs");
    create_dir_all(&certs_path)?;

    let cert_path = certs_path.join(format!("{domain}.pem"));
    let key_path = certs_path.join(format!("{domain}.key"));

    if cert_path.exists() && key_path.exists() {
      let certs = load_certs(&cert_path)?;
      if !certs_need_renewal(&certs)? {
        print_message("The ACME certificate is still valid; no renewal is needed.");
        return Ok(());
      }
      debug!(%domain, "certificate needs renewal");
    }

    let account = Self::acme_account(config).await?;

    let mut order =
      account.new_order(&NewOrder::new(&[Identifier::Dns(domain.into())])).await?;

    match order.state().status {
      OrderStatus::Pending => {
        let mut authorizations = order.authorizations();

        while let Some(auth) = authorizations.next().await {
          let mut authz = auth?;

          match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            status => bail!("unexpected authorization status: {status:?}"),
          }

          let mut challenge = authz
            .challenge(ChallengeType::Dns01)
            .ok_or_else(|| anyhow::anyhow!("no dns01 challenge found"))?;

          let record_name = format!("_acme-challenge.{}", challenge.identifier());
          let challenge_value = challenge.key_authorization().dns_value();

          print_separator(44);
          print_message("Please set the following DNS record then press the Return key:");
          print_field("Name", &record_name);
          print_field("Record type", "TXT");
          print_field("Content", &challenge_value);
          print_separator(44);

          loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            resolver.clear_lookup_cache(&record_name, RecordType::TXT);
            let query = resolver.txt_lookup(&record_name).await?;

            let has_added_record = query.answers().iter().any(|r| match &r.data {
              RData::TXT(txt) => {
                let value = txt
                  .txt_data
                  .iter()
                  .flat_map(|chunk| chunk.iter().copied())
                  .collect::<Vec<u8>>();

                value == challenge_value.as_bytes()
              }
              _ => false,
            });

            if has_added_record {
              challenge.set_ready().await?;
              break;
            }

            error!(
              "the DNS TXT record has still not been added. wait a few minutes before \
               retrying"
            );
            print_message("Press Return to check again.");
          }
        }

        let status = order.poll_ready(&RetryPolicy::default()).await?;
        if status != OrderStatus::Ready {
          bail!("unexpected order status after challenge validation: {status:?}");
        }
      }

      OrderStatus::Ready => {
        debug!(%domain, "ACME order is already ready, skipping challenges");
      }

      status => {
        bail!("unexpected initial order status: {status:?}");
      }
    }

    let status = order.poll_ready(&RetryPolicy::default()).await?;
    if status != OrderStatus::Ready {
      bail!("unexpected order status: {status:?}");
    }

    let private_key_pem = order.finalize().await?;
    let cert_chain_pem = order.poll_certificate(&RetryPolicy::default()).await?;

    let cert_tmp_path = cert_path.with_extension("pem.tmp");
    let key_tmp_path = key_path.with_extension("key.tmp");

    write(&cert_tmp_path, cert_chain_pem)?;
    write(&key_tmp_path, private_key_pem)?;

    if cert_path.exists() {
      remove_file(&cert_path)?;
    }
    if key_path.exists() {
      remove_file(&key_path)?;
    }

    rename(cert_tmp_path, &cert_path)?;
    rename(key_tmp_path, &key_path)?;

    print_success("ACME certificate fetched successfully");
    Ok(())
  }
}
