pub mod items;
pub mod loans;
pub mod images;
pub mod cable_colors;
pub mod labels;
pub mod containers;

pub use items::*;
pub use loans::*;
pub use images::*;
pub use cable_colors::*;
pub use labels::*;
pub use containers::*;

use std::sync::Arc;
use crate::services::{ItemService, LoanService, StorageService, CableColorService};