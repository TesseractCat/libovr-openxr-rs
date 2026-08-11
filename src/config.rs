//! Game-directory configuration for the compatibility shim.

use serde::Deserialize;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
struct FileConfig {
    #[serde(default)]
    user: UserConfig,
}

#[derive(Debug, Default, Deserialize)]
struct UserConfig {
    id: Option<u64>,
    org_id: Option<u64>,
    oculus_id: Option<String>,
}

#[derive(Debug)]
pub struct UserIdentity {
    pub id: u64,
    pub org_id: u64,
    pub oculus_id: CString,
}

fn config_path() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|path| {
        path.parent()
            .map(|parent| parent.join("libovr-openxr.toml"))
    })
}

fn load_identity() -> UserIdentity {
    let config = config_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|contents| toml::from_str::<FileConfig>(&contents).ok())
        .unwrap_or(FileConfig {
            user: UserConfig::default(),
        });
    let name = config
        .user
        .oculus_id
        .unwrap_or_else(|| "OpenXRLocalUser".to_owned());
    let oculus_id = CString::new(name)
        .unwrap_or_else(|_| CString::new("OpenXRLocalUser").expect("literal has no NUL"));
    let id = config.user.id.unwrap_or(1);
    UserIdentity {
        id,
        org_id: config.user.org_id.unwrap_or(id),
        oculus_id,
    }
}

pub fn user_identity() -> &'static UserIdentity {
    static IDENTITY: OnceLock<UserIdentity> = OnceLock::new();
    IDENTITY.get_or_init(load_identity)
}
