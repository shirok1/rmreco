#[cfg(test)]
mod tests;

pub mod proto;

#[cfg(feature = "blocking_client")]
pub mod blocking_client;

#[cfg(feature = "tokio_client")]
pub mod tokio_client;