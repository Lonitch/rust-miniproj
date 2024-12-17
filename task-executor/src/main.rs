use std::sync::Arc;
use task_executor::{generate_payloads, ApiConfig};

#[tokio::main]
async fn main()
{
  println!("Task Executor Project\n");

  let num_payloads = 10;
  let payload_vec = generate_payloads(num_payloads);

  let api_config = Arc::new(ApiConfig::new());

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
  for priority_name in ["High", "Medium", "Low"].iter() {
    println!("\n{} Priority Tasks:", priority_name);
    for result in api_config.metrics
                            .lock()
                            .await
                            .values()
                            .filter(|&r| &r.priority.to_string() == priority_name)
    {
      println!("{}", result);
    }
  }
}
