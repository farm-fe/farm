use create_farm_rs::utils::colors::*;

fn main() {
  let colored_text = format!("{}Hello, {}world!{}", RED, GREEN, RESET);
  println!("{}", colored_text);

  // let text_with_colors = "\x1b[31mHello, \x1b[32mworld!\x1b[0m";
  // let text_without_colors = utils::colors::remove_colors(text_with_colors);
  // println!("Text without colors: {}", text_without_colors);
}
