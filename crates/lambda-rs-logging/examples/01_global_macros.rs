fn main() {
  logging::trace!("trace example");
  logging::debug!("debug example: {}", 42);
  logging::info!("info example");
  logging::warn!("warn example");
  logging::error!("error example");
  logging::fatal!("fatal example (no exit)");
}
