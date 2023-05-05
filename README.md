# Mini Executor

A minimal task executor that runs a single future to completion.

This executor is for educational purposes and is not meant for production use. For a more complete and efficient executor, consider using [Tokio](https://crates.io/crates/tokio) or [async-std](https://crates.io/crates/async-std).

## Example Usage

```rust
async fn fetch_data(source: &str, delay: Duration) -> String {
    sleep(delay);
    format!("Data from source: {}", source)
}

async fn concurrent_fetch() {
    let source1 = "Source 1";
    let source2 = "Source 2";
    let delay1 = Duration::from_millis(1000);
    let delay2 = Duration::from_millis(2000);

    let fetch1 = fetch_data(source1, delay1).boxed();
    let fetch2 = fetch_data(source2, delay2).boxed();

    match join!(fetch1, fetch2) {
        (data1, data2) => {
            println!("Data1: {}\n", data1);
            println!("Data2: {}\n", data2);
        }
    }
}

fn main() {
    let executor = MiniExecutor::new(concurrent_fetch());
    executor.run();
}
```