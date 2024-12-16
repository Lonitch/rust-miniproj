// TODO: Update main to use ApiConfiguration
// - Create ApiConfiguration instance at start
// - Clone Arc for each thread to share metrics
// - Replace direct simulate_api_call with api_config.simulate_api_call

use std::sync::Arc;
use task_executor::{generate_payloads, ApiConfig, Priority};

#[tokio::main]
async fn main()
{
  println!("Task Executor Project");

  let num_payloads = 10;
  let payload_vec = generate_payloads(num_payloads);

  let api_config = Arc::new(ApiConfig::new());

  let mut high_priority = Vec::new();
  let mut medium_priority = Vec::new();
  let mut low_priority = Vec::new();

  for payload in &payload_vec {
    match payload.priority {
      Priority::High => high_priority.push(payload.task_id),
      Priority::Medium => medium_priority.push(payload.task_id),
      Priority::Low => low_priority.push(payload.task_id),
    }
  }

  let mut handles = Vec::new();

  for payload in payload_vec {
    let api_config = Arc::clone(&api_config);
    let handle = tokio::spawn(async move {
      let mut retries = 3;
      let mut result = api_config.simulate_api_call(&payload).await;

      while retries > 0 && result.status == "timeout" {
        retries -= 1;
        println!("Task {} timed out, retrying... {} attempts left",
                 result.task_id, retries);
        result = api_config.simulate_api_call(&payload).await;
      }
    });
    handles.push(handle);
  }

  let _results = tokio::join!(futures::future::join_all(handles)).0;

  println!("\nResults by Priority:");
  for (priority_name, priority_tasks) in
    ["High", "Medium", "Low"].iter()
                             .zip([&high_priority, &medium_priority, &low_priority])
  {
    println!("\n{} Priority Tasks:", priority_name);
    for result in api_config.metrics
                            .lock()
                            .await
                            .iter()
                            .filter(|&r| priority_tasks.contains(&r.task_id))
    {
      println!("{}", result);
    }
  }
}
