use tokio::time::Duration;

/// Represents the priority of a task.
pub enum Priority
{
  High,
  Medium,
  Low,
}

/// Represents the input payload for a task.
pub struct Payload
{
  pub url: String,
  pub task_id: u32,
  pub priority: Priority,
}

pub fn generate_payloads(num: u32) -> Vec<Payload>
{
  let mut arr = vec![];
  for _ in 0..=num {
    let url = format!("https://api.example.com/task/{}",
                      (0..8).map(|_| rand::random::<char>())
                            .collect::<String>());
    let task_id = rand::random::<u32>();
    let sd = rand::random::<f32>();
    let priority = match sd {
      sd if sd < 0.33 => Priority::Low,
      sd if sd < 0.66 => Priority::Medium,
      _ => Priority::High,
    };
    arr.push(Payload { url, task_id, priority });
  }
  arr
}

/// Represents the result of a task.
#[derive(Debug)]
pub struct TaskResult
{
  pub task_id: u32,
  pub status: String,
}

impl std::fmt::Display for TaskResult
{
  fn fmt(&self,
         f: &mut std::fmt::Formatter<'_>)
         -> std::fmt::Result
  {
    write!(f,
           "Task {} - Status: {}",
           self.task_id, self.status)
  }
}
/// Simulates an asynchronous API call based on the task's payload.
pub async fn simulate_api_call(payload: &Payload) -> TaskResult
{
  let base_delay = match payload.priority {
    Priority::High => 1.0,
    Priority::Medium => 2.0,
    Priority::Low => 3.0,
  };
  let sleep_fut = tokio::time::sleep(Duration::from_secs_f32(rand::random::<f32>() * base_delay));
  match tokio::time::timeout(Duration::from_secs_f32(base_delay / 2.0),
                             sleep_fut).await
  {
    Ok(_) => TaskResult { task_id: payload.task_id,
                          status: "done".to_string() },
    Err(_) => TaskResult { task_id: payload.task_id,
                           status: "timeout".to_string() },
  }
}
