This is a repo for mini-projects written in Rust

## Projects
### Enhanced Config Parser with Enums

- Focus: `serde` and `serde_json`
- Objective: Create a program that reads a JSON configuration file, validates it, and generates a summarized output.

#### Steps

1. Define a struct to represent the configuration:
    
    - name (string)
    - version (string)
    - settings (nested object with optional fields like theme and max_connections)
    - features (array of strings)
    - Add an enum: Include a Mode field (enum) with variants like Development, Production, and Testing.

2. Use serde to deserialize the JSON file into the struct, deriving serialization and deserialization for the enum and struct.
3. Implement validation logic:

    - Ensure name and version are non-empty.
    - Provide default values for some optional fields in settings.
    - Validate that Mode is one of the defined enum variants.

4. Serialize the parsed configuration back into JSON, including the Mode, and print it in a formatted style.

#### Bonus 
Write a custom deserialization function for the Mode enum to support case-insensitive parsing.

 
### Concurrent Task Executor with a Structured Payload

- Focus: `tokio`
- Objective: Create a small program that executes a list of tasks concurrently, with each task handling a structured payload.

#### Steps

1. Create a Payload struct to represent each task's input:

- url (string)
- task_id (integer)
- priority (enum with variants like High, Medium, Low).

2. Implement an asynchronous function to simulate an API call. It should:

- Take a Payload as input.
- Simulate processing time with a random delay based on the priority.
- Return a result that includes the task_id and a simulated status.

3. Generate a list of Payload instances, each with random URLs, IDs, and priorities.

4. Use `tokio::spawn` to launch multiple tasks concurrently, each processing a Payload.

5. Collect and display the results, grouping them by priority.

#### Bonus 
- Add a timeout for each task using tokio::time::timeout.
- Implement retries for failed tasks.

## Tips

How do you profile the performance of your tools (CPU/memory)?

- [tracing](https://crates.io/crates/tracing)
- avoid boxing, cloning
- try to use references
- avoid for loops (use iterators as much as possible)
