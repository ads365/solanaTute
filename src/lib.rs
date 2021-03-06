//register all modules here

//register entrypoint module adn the no entrypoint feature
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

//register instruction
pub mod instruction;

//register processor
pub mod processor;