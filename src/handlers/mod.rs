pub mod items;
pub mod loans;
pub mod images;
pub mod cable_colors;

pub use items::*;
pub use loans::*;
pub use images::*;
pub use cable_colors::*;

use std::sync::Arc;
use crate::config::Config;
use crate::services::{ItemService, LoanService, StorageService, CableColorService};

pub type AppState = (Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>, Arc<StorageService>, Arc<Config>);