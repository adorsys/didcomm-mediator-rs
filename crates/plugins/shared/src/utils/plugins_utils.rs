use once_cell::sync::OnceCell;
use std::sync::Arc;

static GLOBAL_PLUGIN_UTILS: OnceCell<Arc<PluginsUtils>> = OnceCell::new();

#[derive(Clone)]
pub struct PluginsUtils {
    pub storage_dirpath: String,
    pub server_public_domain: String,
    pub server_local_port: String,
}

/// Initialize common plugins utils that will be shared by all plugins
///
/// This function will be executed only once. All subsequent calls will return the initilized value.
pub fn initialize_plugins_utils() -> Arc<PluginsUtils> {
    GLOBAL_PLUGIN_UTILS
        .get_or_init(|| {
            let storage_dirpath =
                std::env::var("STORAGE_DIRPATH").expect("STORAGE_DIRPATH env variable required");
            let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN")
                .expect("SERVER_PUBLIC_DOMAIN env variable required");
            let server_local_port = std::env::var("SERVER_LOCAL_PORT")
                .expect("SERVER_LOCAL_PORT env variable required");

            Arc::new(PluginsUtils {
                storage_dirpath,
                server_public_domain,
                server_local_port,
            })
        })
        .clone()
}
