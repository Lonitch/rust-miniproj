use task_executor::{generate_payloads, simulate_api_call, Priority};

#[tokio::main]
async fn main()
{
  println!("Task Executor Project");

  let num_payloads = 10;
  let payload_vec = generate_payloads(num_payloads);

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
    let handle = tokio::spawn(async move {
      let mut retries = 3;
      let mut result = simulate_api_call(&payload).await;

      while retries > 0 && result.status == "timeout" {
        retries -= 1;
        println!("Task {} timed out, retrying... {} attempts left",
                 result.task_id, retries);
        result = simulate_api_call(&payload).await;
      }

      result
    });
    handles.push(handle);
  }

  let results = tokio::join!(futures::future::join_all(handles)).0;

  println!("\nResults by Priority:");
  for (priority_name, priority_tasks) in
    ["High", "Medium", "Low"].iter()
                             .zip([&high_priority, &medium_priority, &low_priority])
  {
    println!("\n{} Priority Tasks:", priority_name);
    for result in results.iter()
                         .filter_map(|r| r.as_ref().ok())
                         .filter(|r| priority_tasks.contains(&r.task_id))
    {
      println!("{}", result);
    }
  }
}
