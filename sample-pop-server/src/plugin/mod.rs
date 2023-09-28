pub mod loader;
pub mod traits;

use lazy_static::lazy_static;
use traits::Plugin;

mod index;

lazy_static! {
    pub static ref PLUGINS: Vec<Box<dyn Plugin>> = vec![
        #[cfg(feature = "plugin.index")]
        Box::<index::IndexPlugin>::default()
    ];
}
