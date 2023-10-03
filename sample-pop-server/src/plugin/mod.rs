pub mod container;
pub mod traits;

use lazy_static::lazy_static;
use traits::Plugin;

#[cfg(feature = "plugin.index")]
mod index;

lazy_static! {
    pub static ref PLUGINS: Vec<Box<dyn Plugin>> = vec![
        #[cfg(feature = "plugin.index")]
        Box::<index::IndexPlugin>::default()
    ];
}
