use std::thread::JoinHandle;

use farmfe_core::parking_lot::Mutex;

/// Owned cache jobs. Callers stop admitting compilation before the final drain.
/// Jobs capture the compilation context, never this queue, so joining while
/// holding the queue lock cannot wait on a job that needs the same lock.
#[derive(Default)]
pub(crate) struct CacheTasks {
  tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl CacheTasks {
  pub fn spawn(&self, job: impl FnOnce() + Send + 'static) -> std::io::Result<()> {
    let mut tasks = self.tasks.lock();
    tasks.push(
      std::thread::Builder::new()
        .name("farm-cache-write".into())
        .spawn(job)?,
    );
    Ok(())
  }

  pub fn drain(&self) -> Result<(), String> {
    let mut tasks = self.tasks.lock();
    let mut failures = Vec::new();
    for task in tasks.drain(..) {
      if let Err(error) = task.join() {
        let message = error
          .downcast_ref::<String>()
          .map(String::as_str)
          .or_else(|| error.downcast_ref::<&str>().copied())
          .unwrap_or("unknown panic");
        failures.push(message.to_string());
      }
    }
    if failures.is_empty() {
      Ok(())
    } else {
      Err(format!(
        "Farm cache write task failed: {}",
        failures.join("; ")
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::{Arc, mpsc};
  use std::time::Duration;

  #[test]
  fn drain_waits_for_all_owned_jobs_and_can_be_repeated() {
    let tasks = Arc::new(CacheTasks::default());
    let (release, gate) = mpsc::channel();
    let (entered, started) = mpsc::channel();
    tasks
      .spawn(move || {
        entered.send(()).unwrap();
        gate.recv().unwrap();
      })
      .unwrap();
    started.recv().unwrap();
    let (done, completed) = mpsc::channel();
    let draining = tasks.clone();
    let waiter = std::thread::spawn(move || {
      done.send(draining.drain()).unwrap();
    });
    assert!(completed.recv_timeout(Duration::from_millis(30)).is_err());
    release.send(()).unwrap();
    completed
      .recv_timeout(Duration::from_secs(5))
      .unwrap()
      .unwrap();
    waiter.join().unwrap();
    tasks.drain().unwrap();
  }

  #[test]
  fn panic_does_not_skip_other_jobs() {
    let tasks = CacheTasks::default();
    tasks.spawn(|| panic!("cache failure")).unwrap();
    let (done, completed) = mpsc::channel();
    tasks
      .spawn(move || {
        done.send(()).unwrap();
      })
      .unwrap();
    assert!(tasks.drain().unwrap_err().contains("cache failure"));
    completed.recv().unwrap();
    tasks.drain().unwrap();
  }
}
