//! Preserve upstream embedded asset notices in the distributed native package.
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let destination = std::env::args_os()
    .nth(1)
    .ok_or("Usage: acknowledgements <output.md>")?;
  let listing = two_face::acknowledgement::listing();
  let mut output = format!(
    "# Embedded syntax and theme acknowledgements\n\nGenerated from two-face 0.5.2+bat-0.26.1.\n\nUpstream listing: {}\n\n",
    two_face::acknowledgement::url()
  );
  for license in listing.for_syntaxes().iter().chain(listing.for_themes()) {
    license.write_md(&mut output);
  }
  std::fs::write(destination, output)?;
  Ok(())
}
