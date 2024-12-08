#[derive(Debug)]
pub enum Priority
{
  High,
  Medium,
  Low,
}

#[derive(Debug)]
pub struct Payload
{
  url: String,
  task_id: i32,
  priority: Priority,
}

#[tokio::main]
async fn main()
{
  println!("Task Executor Project");
}
