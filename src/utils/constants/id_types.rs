// src/utils/constants/id_types.rs
use uuid::Uuid;
pub type ObjectId = Uuid;
pub type PlayerId = usize; // usize matches to machine word size, so it should be the same as the size of a pointer