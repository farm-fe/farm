use eframe::egui;
use std::sync::Arc;

pub struct ProfileApp {
  frame_counter: u64,
  compiler: Arc<farmfe_compiler::Compiler>,
}

impl ProfileApp {
  pub fn new(compiler: Arc<farmfe_compiler::Compiler>) -> Self {
    Self {
      frame_counter: 0,
      compiler,
    }
  }
}

impl eframe::App for ProfileApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    farmfe_core::puffin::GlobalProfiler::lock().new_frame(); // call once per frame!

    puffin_egui::profiler_window(ctx);

    if self.frame_counter == 0 {
      self.compiler.compile().unwrap();
    }

    self.frame_counter = 1;
  }
}
