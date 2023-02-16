// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Keylime Authors

use crate::{error::Error, permissions, tpm};
use config::{
    builder::DefaultState, Config, ConfigBuilder, ConfigError, Environment,
    File, FileFormat, Map, Source, Value,
};
use glob::glob;
use keylime::algorithms::{
    EncryptionAlgorithm, HashAlgorithm, SignAlgorithm,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub static CONFIG_VERSION: &str = "2.0";
pub static DEFAULT_UUID: &str = "d432fbb3-d2f1-4a97-9ef7-75bd81c00000";
pub static DEFAULT_IP: &str = "127.0.0.1";
pub static DEFAULT_PORT: u32 = 9002;
pub static DEFAULT_CONTACT_IP: &str = "127.0.0.1";
pub static DEFAULT_CONTACT_PORT: u32 = 9002;
pub static DEFAULT_REGISTRAR_IP: &str = "127.0.0.1";
pub static DEFAULT_REGISTRAR_PORT: u32 = 8890;
pub static DEFAULT_ENABLE_AGENT_MTLS: bool = true;
pub static DEFAULT_KEYLIME_DIR: &str = "/var/lib/keylime";
pub static DEFAULT_SERVER_KEY: &str = "server-private.pem";
pub static DEFAULT_SERVER_CERT: &str = "server-cert.crt";
pub static DEFAULT_SERVER_KEY_PASSWORD: &str = "";
// The DEFAULT_TRUSTED_CLIENT_CA is relative from KEYLIME_DIR
pub static DEFAULT_TRUSTED_CLIENT_CA: &str = "cv_ca/cacert.crt";
pub static DEFAULT_ENC_KEYNAME: &str = "derived_tci_key";
pub static DEFAULT_DEC_PAYLOAD_FILE: &str = "decrypted_payload";
pub static DEFAULT_SECURE_SIZE: &str = "1m";
pub static DEFAULT_TPM_OWNERPASSWORD: &str = "";
pub static DEFAULT_EXTRACT_PAYLOAD_ZIP: bool = true;
pub static DEFAULT_ENABLE_REVOCATION_NOTIFICATIONS: bool = false;
pub static DEFAULT_REVOCATION_ACTIONS_DIR: &str = "/usr/libexec/keylime";
pub static DEFAULT_REVOCATION_NOTIFICATION_IP: &str = "127.0.0.1";
pub static DEFAULT_REVOCATION_NOTIFICATION_PORT: u32 = 8992;
// Note: The revocation certificate name is generated inside the Python tenant and the
// certificate(s) can be generated by running the tenant with the --cert flag. For more
// information, check the README: https://github.com/keylime/keylime/#using-keylime-ca
pub static DEFAULT_REVOCATION_CERT: &str = "RevocationNotifier-cert.crt";
pub static DEFAULT_REVOCATION_ACTIONS: &str = "";
pub static DEFAULT_PAYLOAD_SCRIPT: &str = "autorun.sh";
pub static DEFAULT_ENABLE_INSECURE_PAYLOAD: bool = false;
pub static DEFAULT_ALLOW_PAYLOAD_REVOCATION_ACTIONS: bool = true;
pub static DEFAULT_TPM_HASH_ALG: &str = "sha256";
pub static DEFAULT_TPM_ENCRYPTION_ALG: &str = "rsa";
pub static DEFAULT_TPM_SIGNING_ALG: &str = "rsassa";
pub static DEFAULT_EK_HANDLE: &str = "generate";
pub static DEFAULT_RUN_AS: &str = "keylime:tss";
pub static DEFAULT_AGENT_DATA_PATH: &str = "agent_data.json";
pub static DEFAULT_CONFIG: &str = "/etc/keylime/agent.conf";
pub static DEFAULT_CONFIG_SYS: &str = "/usr/etc/keylime/agent.conf";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EnvConfig {
    pub version: Option<String>,
    pub uuid: Option<String>,
    pub ip: Option<String>,
    pub port: Option<u32>,
    pub contact_ip: Option<String>,
    pub contact_port: Option<u32>,
    pub registrar_ip: Option<String>,
    pub registrar_port: Option<u32>,
    pub enable_agent_mtls: Option<bool>,
    pub keylime_dir: Option<String>,
    pub server_key: Option<String>,
    pub server_cert: Option<String>,
    pub server_key_password: Option<String>,
    pub trusted_client_ca: Option<String>,
    pub enc_keyname: Option<String>,
    pub dec_payload_file: Option<String>,
    pub secure_size: Option<String>,
    pub tpm_ownerpassword: Option<String>,
    pub extract_payload_zip: Option<bool>,
    pub enable_revocation_notifications: Option<bool>,
    pub revocation_actions_dir: Option<String>,
    pub revocation_notification_ip: Option<String>,
    pub revocation_notification_port: Option<u32>,
    pub revocation_cert: Option<String>,
    pub revocation_actions: Option<String>,
    pub payload_script: Option<String>,
    pub enable_insecure_payload: Option<bool>,
    pub allow_payload_revocation_actions: Option<bool>,
    pub tpm_hash_alg: Option<String>,
    pub tpm_encryption_alg: Option<String>,
    pub tpm_signing_alg: Option<String>,
    pub ek_handle: Option<String>,
    pub run_as: Option<String>,
    pub agent_data_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct AgentConfig {
    pub version: String,
    pub uuid: String,
    pub ip: String,
    pub port: u32,
    pub contact_ip: String,
    pub contact_port: u32,
    pub registrar_ip: String,
    pub registrar_port: u32,
    pub enable_agent_mtls: bool,
    pub keylime_dir: String,
    pub server_key: String,
    pub server_cert: String,
    pub server_key_password: String,
    pub trusted_client_ca: String,
    pub enc_keyname: String,
    pub dec_payload_file: String,
    pub secure_size: String,
    pub tpm_ownerpassword: String,
    pub extract_payload_zip: bool,
    pub enable_revocation_notifications: bool,
    pub revocation_actions_dir: String,
    pub revocation_notification_ip: String,
    pub revocation_notification_port: u32,
    pub revocation_cert: String,
    pub revocation_actions: String,
    pub payload_script: String,
    pub enable_insecure_payload: bool,
    pub allow_payload_revocation_actions: bool,
    pub tpm_hash_alg: String,
    pub tpm_encryption_alg: String,
    pub tpm_signing_alg: String,
    pub ek_handle: String,
    pub run_as: String,
    pub agent_data_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct KeylimeConfig {
    pub agent: AgentConfig,
}

impl EnvConfig {
    pub fn get_map(&self) -> Map<String, Value> {
        let mut agent: Map<String, Value> = Map::new();
        if let Some(ref v) = self.version {
            _ = agent.insert("version".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.uuid {
            _ = agent.insert("uuid".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.ip {
            _ = agent.insert("ip".to_string(), v.to_string().into());
        }
        if let Some(v) = self.port {
            _ = agent.insert("port".to_string(), v.into());
        }
        if let Some(ref v) = self.contact_ip {
            _ = agent.insert("contact_ip".to_string(), v.to_string().into());
        }
        if let Some(v) = self.contact_port {
            _ = agent.insert("contact_port".to_string(), v.into());
        }
        if let Some(ref v) = self.registrar_ip {
            _ = agent
                .insert("registrar_ip".to_string(), v.to_string().into());
        }
        if let Some(v) = self.registrar_port {
            _ = agent.insert("registrar_port".to_string(), v.into());
        }
        if let Some(v) = self.enable_agent_mtls {
            _ = agent.insert("enable_agent_mtls".to_string(), v.into());
        }
        if let Some(ref v) = self.keylime_dir {
            _ = agent.insert("keylime_dir".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.server_key {
            _ = agent.insert("server_key".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.server_key_password {
            _ = agent.insert(
                "server_key_password".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(ref v) = self.server_cert {
            _ = agent.insert("server_cert".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.trusted_client_ca {
            _ = agent.insert(
                "trusted_client_ca".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(ref v) = self.enc_keyname {
            _ = agent.insert("enc_keyname".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.dec_payload_file {
            _ = agent
                .insert("dec_payload_file".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.secure_size {
            _ = agent.insert("secure_size".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.tpm_ownerpassword {
            _ = agent.insert(
                "tpm_ownerpassword".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(v) = self.extract_payload_zip {
            _ = agent.insert("extract_payload_zip".to_string(), v.into());
        }
        if let Some(v) = self.enable_revocation_notifications {
            _ = agent.insert(
                "enable_revocation_notifications".to_string(),
                v.into(),
            );
        }
        if let Some(ref v) = self.revocation_actions_dir {
            _ = agent.insert(
                "revocation_actions_dir".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(ref v) = self.revocation_notification_ip {
            _ = agent.insert(
                "revocation_notification_ip".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(v) = self.revocation_notification_port {
            _ = agent
                .insert("revocation_notification_port".to_string(), v.into());
        }
        if let Some(ref v) = self.revocation_cert {
            _ = agent
                .insert("revocation_cert".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.revocation_actions {
            _ = agent.insert(
                "revocation_actions".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(ref v) = self.payload_script {
            _ = agent
                .insert("payload_script".to_string(), v.to_string().into());
        }
        if let Some(v) = self.enable_insecure_payload {
            _ = agent.insert("enable_insecure_payload".to_string(), v.into());
        }
        if let Some(v) = self.allow_payload_revocation_actions {
            _ = agent.insert(
                "allow_payload_revocation_actions".to_string(),
                v.into(),
            );
        }
        if let Some(ref v) = self.tpm_hash_alg {
            _ = agent
                .insert("tpm_hash_alg".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.tpm_encryption_alg {
            _ = agent.insert(
                "tpm_encryption_alg".to_string(),
                v.to_string().into(),
            );
        }
        if let Some(ref v) = self.tpm_signing_alg {
            _ = agent
                .insert("tpm_signing_alg".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.ek_handle {
            _ = agent.insert("ek_handle".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.run_as {
            _ = agent.insert("run_as".to_string(), v.to_string().into());
        }
        if let Some(ref v) = self.agent_data_path {
            _ = agent
                .insert("agent_data_path".to_string(), v.to_string().into());
        }
        agent
    }

    pub fn iter(&self) -> impl Iterator {
        self.get_map().into_iter()
    }
}

impl KeylimeConfig {
    pub fn new() -> Result<Self, Error> {
        // Get the base configuration file from the environment variable or the default locations
        let setting = config_get_setting()?.build()?;
        let config: KeylimeConfig = setting.try_deserialize()?;

        // Replace keywords with actual values
        config_translate_keywords(&config)
    }
}

impl Source for EnvConfig {
    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        // Note: the returned mapping matches the KeylimeConfig
        // This is to allow overriding the values using environment variables
        Ok(Map::from([("agent".to_string(), self.get_map().into())]))
    }

    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Source for KeylimeConfig {
    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut m: Map<String, Value> = Map::new();

        _ = m.insert(
            "version".to_string(),
            self.agent.version.to_string().into(),
        );
        _ = m.insert("uuid".to_string(), self.agent.uuid.to_string().into());
        _ = m.insert("ip".to_string(), self.agent.ip.to_string().into());
        _ = m.insert("port".to_string(), self.agent.port.into());
        _ = m.insert(
            "contact_ip".to_string(),
            self.agent.contact_ip.to_string().into(),
        );
        _ = m.insert(
            "contact_port".to_string(),
            self.agent.contact_port.into(),
        );
        _ = m.insert(
            "registrar_ip".to_string(),
            self.agent.registrar_ip.to_string().into(),
        );
        _ = m.insert(
            "registrar_port".to_string(),
            self.agent.registrar_port.into(),
        );
        _ = m.insert(
            "enable_agent_mtls".to_string(),
            self.agent.enable_agent_mtls.into(),
        );
        _ = m.insert(
            "keylime_dir".to_string(),
            self.agent.keylime_dir.to_string().into(),
        );
        _ = m.insert(
            "server_key".to_string(),
            self.agent.server_key.to_string().into(),
        );
        _ = m.insert(
            "server_key_password".to_string(),
            self.agent.server_key_password.to_string().into(),
        );
        _ = m.insert(
            "server_cert".to_string(),
            self.agent.server_cert.to_string().into(),
        );
        _ = m.insert(
            "trusted_client_ca".to_string(),
            self.agent.trusted_client_ca.to_string().into(),
        );
        _ = m.insert(
            "enc_keyname".to_string(),
            self.agent.enc_keyname.to_string().into(),
        );
        _ = m.insert(
            "dec_payload_file".to_string(),
            self.agent.dec_payload_file.to_string().into(),
        );
        _ = m.insert(
            "secure_size".to_string(),
            self.agent.secure_size.to_string().into(),
        );
        _ = m.insert(
            "tpm_ownerpassword".to_string(),
            self.agent.tpm_ownerpassword.to_string().into(),
        );
        _ = m.insert(
            "extract_payload_zip".to_string(),
            self.agent.extract_payload_zip.to_string().into(),
        );
        _ = m.insert(
            "enable_revocation_notifications".to_string(),
            self.agent
                .enable_revocation_notifications
                .to_string()
                .into(),
        );
        _ = m.insert(
            "revocation_actions_dir".to_string(),
            self.agent.revocation_actions_dir.to_string().into(),
        );
        _ = m.insert(
            "revocation_notification_ip".to_string(),
            self.agent.revocation_notification_ip.to_string().into(),
        );
        _ = m.insert(
            "revocation_notification_port".to_string(),
            self.agent.revocation_notification_port.into(),
        );
        _ = m.insert(
            "revocation_cert".to_string(),
            self.agent.revocation_cert.to_string().into(),
        );
        _ = m.insert(
            "revocation_actions".to_string(),
            self.agent.revocation_actions.to_string().into(),
        );
        _ = m.insert(
            "payload_script".to_string(),
            self.agent.payload_script.to_string().into(),
        );
        _ = m.insert(
            "enable_insecure_payload".to_string(),
            self.agent.enable_insecure_payload.into(),
        );
        _ = m.insert(
            "allow_payload_revocation_actions".to_string(),
            self.agent.allow_payload_revocation_actions.into(),
        );
        _ = m.insert(
            "tpm_hash_alg".to_string(),
            self.agent.tpm_hash_alg.to_string().into(),
        );
        _ = m.insert(
            "tpm_encryption_alg".to_string(),
            self.agent.tpm_encryption_alg.to_string().into(),
        );
        _ = m.insert(
            "tpm_signing_alg".to_string(),
            self.agent.tpm_signing_alg.to_string().into(),
        );
        _ = m.insert(
            "ek_handle".to_string(),
            self.agent.ek_handle.to_string().into(),
        );
        _ = m.insert(
            "run_as".to_string(),
            self.agent.run_as.to_string().into(),
        );
        _ = m.insert(
            "agent_data_path".to_string(),
            self.agent.agent_data_path.to_string().into(),
        );

        Ok(Map::from([("agent".to_string(), m.into())]))
    }

    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        // In case the process is executed by privileged user
        let run_as = if permissions::get_euid() == 0 {
            DEFAULT_RUN_AS.to_string()
        } else {
            "".to_string()
        };

        AgentConfig {
            version: CONFIG_VERSION.to_string(),
            ip: DEFAULT_IP.to_string(),
            port: DEFAULT_PORT,
            registrar_ip: DEFAULT_REGISTRAR_IP.to_string(),
            registrar_port: DEFAULT_REGISTRAR_PORT,
            uuid: DEFAULT_UUID.to_string(),
            contact_ip: DEFAULT_CONTACT_IP.to_string(),
            contact_port: DEFAULT_CONTACT_PORT,
            tpm_hash_alg: DEFAULT_TPM_HASH_ALG.to_string(),
            tpm_encryption_alg: DEFAULT_TPM_ENCRYPTION_ALG.to_string(),
            tpm_signing_alg: DEFAULT_TPM_SIGNING_ALG.to_string(),
            agent_data_path: "default".to_string(),
            enable_revocation_notifications:
                DEFAULT_ENABLE_REVOCATION_NOTIFICATIONS,
            revocation_cert: "default".to_string(),
            revocation_notification_ip: DEFAULT_REVOCATION_NOTIFICATION_IP
                .to_string(),
            revocation_notification_port:
                DEFAULT_REVOCATION_NOTIFICATION_PORT,
            secure_size: DEFAULT_SECURE_SIZE.to_string(),
            payload_script: DEFAULT_PAYLOAD_SCRIPT.to_string(),
            dec_payload_file: DEFAULT_DEC_PAYLOAD_FILE.to_string(),
            enc_keyname: DEFAULT_ENC_KEYNAME.to_string(),
            extract_payload_zip: DEFAULT_EXTRACT_PAYLOAD_ZIP,
            server_key: "default".to_string(),
            server_key_password: DEFAULT_SERVER_KEY_PASSWORD.to_string(),
            server_cert: "default".to_string(),
            trusted_client_ca: "default".to_string(),
            revocation_actions: DEFAULT_REVOCATION_ACTIONS.to_string(),
            revocation_actions_dir: DEFAULT_REVOCATION_ACTIONS_DIR
                .to_string(),
            allow_payload_revocation_actions:
                DEFAULT_ALLOW_PAYLOAD_REVOCATION_ACTIONS,
            keylime_dir: DEFAULT_KEYLIME_DIR.to_string(),
            enable_agent_mtls: DEFAULT_ENABLE_AGENT_MTLS,
            enable_insecure_payload: DEFAULT_ENABLE_INSECURE_PAYLOAD,
            run_as,
            tpm_ownerpassword: DEFAULT_TPM_OWNERPASSWORD.to_string(),
            ek_handle: DEFAULT_EK_HANDLE.to_string(),
        }
    }
}

impl Default for KeylimeConfig {
    fn default() -> Self {
        let c = KeylimeConfig {
            agent: AgentConfig::default(),
        };

        // The default config should never fail to translate keywords
        config_translate_keywords(&c).unwrap() //#[allow_ci]
    }
}

fn config_get_env_setting() -> Result<impl Source, Error> {
    let env_config: EnvConfig = Config::builder()
        // Add environment variables overrides
        .add_source(
            Environment::with_prefix("KEYLIME_AGENT")
                .separator(".")
                .prefix_separator("_"),
        )
        .build()?
        .try_deserialize()?;

    // Log debug message for configuration obtained from environment
    env_config
        .get_map()
        .iter()
        .for_each(|(c, v)| debug!("Environment configuration {c}={v}"));

    Ok(env_config)
}

fn config_get_file_setting() -> Result<ConfigBuilder<DefaultState>, Error> {
    let default_config = KeylimeConfig::default();

    Ok(Config::builder()
        // Default values
        .add_source(default_config)
        // Add system configuration file
        .add_source(
            File::new(DEFAULT_CONFIG_SYS, FileFormat::Toml).required(false),
        )
        // Add system configuration snippets
        .add_source(
            glob("/usr/etc/keylime/agent.conf.d/*")
                .map_err(Error::GlobPattern)?
                .filter_map(|entry| entry.ok())
                .map(|path| {
                    File::new(&path.display().to_string(), FileFormat::Toml)
                        .required(false)
                })
                .collect::<Vec<_>>(),
        )
        .add_source(
            File::new(DEFAULT_CONFIG, FileFormat::Toml).required(false),
        )
        // Add user configuration snippets
        .add_source(
            glob("/etc/keylime/agent.conf.d/*")
                .map_err(Error::GlobPattern)?
                .filter_map(|entry| entry.ok())
                .map(|path| {
                    File::new(&path.display().to_string(), FileFormat::Toml)
                        .required(false)
                })
                .collect::<Vec<_>>(),
        )
        // Add environment variables overrides
        .add_source(config_get_env_setting()?))
}

fn config_get_setting() -> Result<ConfigBuilder<DefaultState>, Error> {
    if let Ok(env_cfg) = env::var("KEYLIME_AGENT_CONFIG") {
        if !env_cfg.is_empty() {
            let path = Path::new(&env_cfg);
            if (path.exists()) {
                return Ok(Config::builder()
                    .add_source(
                        File::new(&env_cfg, FileFormat::Toml).required(true),
                    )
                    // Add environment variables overrides
                    .add_source(config_get_env_setting()?));
            } else {
                warn!("Configuration set in KEYLIME_AGENT_CONFIG environment variable not found");
                return Err(Error::Configuration("Configuration set in KEYLIME_AGENT_CONFIG environment variable not found".to_string()));
            }
        }
    }
    config_get_file_setting()
}

/// Replace the options that support keywords with the final value
fn config_translate_keywords(
    config: &KeylimeConfig,
) -> Result<KeylimeConfig, Error> {
    let uuid = get_uuid(&config.agent.uuid);

    let env_keylime_dir = env::var("KEYLIME_DIR").ok();
    let keylime_dir = match env_keylime_dir {
        Some(ref dir) => {
            if dir.is_empty() {
                match &config.agent.keylime_dir {
                    s => s.clone(),
                    _ => DEFAULT_KEYLIME_DIR.to_string(),
                }
            } else {
                dir.to_string()
            }
        }
        None => match &config.agent.keylime_dir {
            s => s.clone(),
            _ => DEFAULT_KEYLIME_DIR.to_string(),
        },
    };

    // Validate that keylime_dir exists
    let keylime_dir = Path::new(&keylime_dir).canonicalize().map_err(|e| {
        Error::Configuration(format!(
            "Path {keylime_dir} set in keylime_dir configuration option not found: {e}"
        ))
    })?;

    let mut agent_data_path = config_get_file_path(
        "agent_data_path",
        &config.agent.agent_data_path,
        &keylime_dir,
        DEFAULT_AGENT_DATA_PATH,
    );

    let mut server_key = config_get_file_path(
        "server_key",
        &config.agent.server_key,
        &keylime_dir,
        DEFAULT_SERVER_KEY,
    );

    let mut server_cert = config_get_file_path(
        "server_cert",
        &config.agent.server_cert,
        &keylime_dir,
        DEFAULT_SERVER_CERT,
    );

    let mut trusted_client_ca = config_get_file_path(
        "trusted_client_ca",
        &config.agent.trusted_client_ca,
        &keylime_dir,
        DEFAULT_TRUSTED_CLIENT_CA,
    );

    let ek_handle = match config.agent.ek_handle.as_ref() {
        "generate" => "".to_string(),
        "" => "".to_string(),
        s => s.to_string(),
    };

    // Validate the configuration

    // If revocation notifications is enabled, verify all the required options for revocation
    if config.agent.enable_revocation_notifications {
        if config.agent.revocation_notification_ip.is_empty() {
            error!("The option 'enable_revocation_notifications' is set as 'true' but 'revocation_notification_ip' was set as empty");
            return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but 'revocation_notification_ip' was set as empty".to_string()));
        }
        if config.agent.revocation_cert.is_empty() {
            error!("The option 'enable_revocation_notifications' is set as 'true' 'revocation_cert' was set as empty");
            return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but 'revocation_notification_cert' was set as empty".to_string()));
        }
        let actions_dir = match config.agent.revocation_actions_dir.as_ref() {
            "" => {
                error!("The option 'enable_revocation_notifications' is set as 'true' but the revocation actions directory was set as empty in 'revocation_actions_dir'");
                return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but 'revocation_actions_dir' was set as empty".to_string()));
            }
            dir => dir.to_string(),
        };
    }

    let mut revocation_cert = config_get_file_path(
        "revocation_cert",
        &config.agent.revocation_cert,
        &keylime_dir,
        &format!("secure/unzipped/{DEFAULT_REVOCATION_CERT}"),
    );

    Ok(KeylimeConfig {
        agent: AgentConfig {
            keylime_dir: keylime_dir.display().to_string(),
            uuid,
            server_key,
            server_cert,
            trusted_client_ca,
            ek_handle,
            agent_data_path,
            revocation_cert,
            ..config.agent.clone()
        },
    })
}

/// Expand a file path from the configuration file.
///
/// If the string is set as "default", return the provided default path relative from the provided work_dir.
/// If the string is empty, use again the default value
/// If the string is a relative path, return the path relative from the provided work_dir
/// If the string is an absolute path, return the path without change.
fn config_get_file_path(
    option: &str,
    path: &str,
    work_dir: &Path,
    default: &str,
) -> String {
    match path {
        "default" => work_dir.join(default).display().to_string(),
        "" => {
            warn!("Empty string provided in configuration option {option}, using default {default}");
            work_dir.join(default).display().to_string()
        }
        v => {
            let p = Path::new(v);
            if p.is_relative() {
                work_dir.join(p).display().to_string()
            } else {
                p.display().to_string()
            }
        }
    }
}

fn get_uuid(agent_uuid_config: &str) -> String {
    match agent_uuid_config {
        "hash_ek" => {
            info!("Using hashed EK as UUID");
            // DO NOT change this to something else. It is used later to set the correct value.
            "hash_ek".into()
        }
        "generate" => {
            let agent_uuid = Uuid::new_v4();
            info!("Generated a new UUID: {}", &agent_uuid);
            agent_uuid.to_string()
        }
        uuid_config => match Uuid::parse_str(uuid_config) {
            Ok(uuid_config) => uuid_config.to_string(),
            Err(_) => {
                warn!("Misformatted UUID: {}", &uuid_config);
                let agent_uuid = Uuid::new_v4();
                info!("Using generated UUID: {}", &uuid_config);
                agent_uuid.to_string()
            }
        },
    }
}

// Unit Testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let default = KeylimeConfig::default();

        let c = KeylimeConfig {
            agent: AgentConfig::default(),
        };

        let result = config_translate_keywords(&c);
        assert!(result.is_ok());
        let expected = result.unwrap(); //#[allow_ci]
        assert_eq!(expected, default);
    }

    #[test]
    fn get_revocation_cert_path_default() {
        let test_config = KeylimeConfig::default();
        let revocation_cert_path = test_config.agent.revocation_cert.clone();
        let mut expected = Path::new(&test_config.agent.keylime_dir)
            .join("secure/unzipped")
            .join(DEFAULT_REVOCATION_CERT)
            .display()
            .to_string();
        assert_eq!(revocation_cert_path, expected);
    }

    #[test]
    fn get_revocation_cert_path_absolute() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                revocation_cert: "/test/cert.crt".to_string(),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        let revocation_cert_path = test_config.agent.revocation_cert;
        let mut expected = Path::new("/test/cert.crt").display().to_string();
        assert_eq!(revocation_cert_path, expected);
    }

    #[test]
    fn get_revocation_cert_path_relative() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                revocation_cert: "cert.crt".to_string(),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        let revocation_cert_path = test_config.agent.revocation_cert.clone();
        let mut expected = Path::new(&test_config.agent.keylime_dir)
            .join("cert.crt")
            .display()
            .to_string();
        assert_eq!(revocation_cert_path, expected);
    }

    #[test]
    fn get_revocation_notification_ip_empty() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: true,
                revocation_notification_ip: "".to_string(),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        // Due to enable_revocation_notifications being set
        assert!(result.is_err());
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: false,
                revocation_notification_ip: "".to_string(),
                ..Default::default()
            },
        };

        // Now unset enable_revocation_notifications and check that is allowed
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        assert_eq!(
            test_config.agent.revocation_notification_ip,
            "".to_string()
        );
    }

    #[test]
    fn get_revocation_cert_empty() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: true,
                revocation_cert: "".to_string(),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        // Due to enable_revocation_notifications being set
        assert!(result.is_err());
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: false,
                revocation_cert: "".to_string(),
                ..Default::default()
            },
        };

        // Now unset enable_revocation_notifications and check that is allowed
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
    }

    #[test]
    fn get_revocation_actions_dir_empty() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: true,
                revocation_actions_dir: "".to_string(),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        // Due to enable_revocation_notifications being set
        assert!(result.is_err());
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: false,
                revocation_actions_dir: "".to_string(),
                ..Default::default()
            },
        };

        // Now unset enable_revocation_notifications and check that is allowed
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_uuid() {
        assert_eq!(get_uuid("hash_ek"), "hash_ek");
        let _ = Uuid::parse_str(&get_uuid("generate")).unwrap(); //#[allow_ci]
        assert_eq!(
            get_uuid("D432FBB3-D2F1-4A97-9EF7-75BD81C00000"),
            "d432fbb3-d2f1-4a97-9ef7-75bd81c00000"
        );
        assert_ne!(
            get_uuid("D432FBB3-D2F1-4A97-9EF7-75BD81C0000X"),
            "d432fbb3-d2f1-4a97-9ef7-75bd81c0000X"
        );
        let _ = Uuid::parse_str(&get_uuid(
            "D432FBB3-D2F1-4A97-9EF7-75BD81C0000X",
        ))
        .unwrap(); //#[allow_ci]
    }

    #[test]
    fn test_env_var() {
        let override_map: Map<&str, &str> = Map::from([
            ("VERSION", "override_version"),
            ("UUID", "override_uuid"),
            ("IP", "override_ip"),
            ("PORT", "9999"),
            ("CONTACT_IP", "override_contact_ip"),
            ("CONTACT_PORT", "9999"),
            ("REGISTRAR_IP", "override_registrar_ip"),
            ("REGISTRAR_PORT", "9999"),
            ("ENABLE_AGENT_MTLS", "false"),
            ("KEYLIME_DIR", "override_keylime_dir"),
            ("SERVER_KEY", "override_server_key"),
            ("SERVER_CERT", "override_server_cert"),
            ("SERVER_KEY_PASSWORD", "override_server_key_password"),
            ("TRUSTED_CLIENT_CA", "override_trusted_client_ca"),
            ("ENC_KEYNAME", "override_enc_keyname"),
            ("DEC_PAYLOAD_FILE", "override_dec_payload_file"),
            ("SECURE_SIZE", "override_secure_size"),
            ("TPM_OWNERPASSWORD", "override_tpm_ownerpassword"),
            ("EXTRACT_PAYLOAD_ZIP", "false"),
            ("ENABLE_REVOCATION_NOTIFICATIONS", "false"),
            ("REVOCATION_ACTIONS_DIR", "override_revocation_actions_dir"),
            (
                "REVOCATION_NOTIFICATION_IP",
                "override_revocation_notification_ip",
            ),
            ("REVOCATION_NOTIFICATION_PORT", "9999"),
            ("REVOCATION_CERT", "override_revocation_cert"),
            ("REVOCATION_ACTIONS", "override_revocation_actions"),
            ("PAYLOAD_SCRIPT", "override_payload_script"),
            ("ENABLE_INSECURE_PAYLOAD", "true"),
            ("ALLOW_PAYLOAD_REVOCATION_ACTIONS", "false"),
            ("TPM_HASH_ALG", "override_tpm_hash_alg"),
            ("TPM_ENCRYPTION_ALG", "override_tpm_encryption_alg"),
            ("TPM_SIGNING_ALG", "override_tpm_signing_alg"),
            ("EK_HANDLE", "override_ek_handle"),
            ("RUN_AS", "override_run_as"),
            ("AGENT_DATA_PATH", "override_agent_data_path"),
        ]);

        for (c, v) in override_map.into_iter() {
            let default = KeylimeConfig::default();

            let env_conf: EnvConfig = Config::builder()
                .add_source(
                    Environment::default()
                        .separator(".")
                        .prefix_separator("_")
                        .source(Some({
                            let mut env = Map::new();
                            _ = env.insert(c.into(), v.into());
                            env
                        })),
                )
                .build()
                .unwrap() //#[allow_ci]
                .try_deserialize()
                .unwrap(); //#[allow_ci]

            let new_conf: KeylimeConfig = Config::builder()
                .add_source(default)
                .add_source(env_conf)
                .build()
                .unwrap() //#[allow_ci]
                .try_deserialize()
                .unwrap(); //#[allow_ci]

            let m = new_conf.collect().unwrap(); //#[allow_ci]
            let internal = m.get("agent").unwrap(); //#[allow_ci]
            let obtained = internal.to_owned().into_table().unwrap(); //#[allow_ci]

            let d = KeylimeConfig::default().collect().unwrap(); //#[allow_ci]
            let i = d.get("agent").unwrap(); //#[allow_ci]
            let mut expected = i.to_owned().into_table().unwrap(); //#[allow_ci]
            _ = expected.insert(c.to_lowercase(), v.into());

            for (i, j) in obtained.iter() {
                let e = expected.get(i).unwrap(); //#[allow_ci]
                assert!(e.to_string() == j.to_string());
            }
        }
    }
}
