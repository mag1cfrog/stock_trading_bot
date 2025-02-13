use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use deltalake::arrow::compute::concat_batches;
use deltalake::arrow::{
    array::{Int32Array, StringArray, TimestampMicrosecondArray},
    datatypes::{DataType as ArrowDataType, Field, Schema as ArrowSchema, TimeUnit},
    record_batch::RecordBatch,
};
use deltalake::datafusion::prelude::SessionContext;
use deltalake::kernel::{DataType, PrimitiveType, StructField};
use deltalake::operations::collect_sendable_stream;
use deltalake::protocol::SaveMode;
use deltalake::{open_table, DeltaOps};

// Define the schema including 'writer_id'
fn get_test_table_columns() -> Vec<StructField> {
    vec![
        StructField::new(
            "int".to_string(),
            DataType::Primitive(PrimitiveType::Integer),
            false,
        ),
        StructField::new(
            "string".to_string(),
            DataType::Primitive(PrimitiveType::String),
            true,
        ),
        StructField::new(
            "timestamp".to_string(),
            DataType::Primitive(PrimitiveType::TimestampNtz),
            true,
        ),
        StructField::new(
            "writer_id".to_string(),
            DataType::Primitive(PrimitiveType::Integer),
            false,
        ),
    ]
}

// Generate a RecordBatch with the given writer_id
fn generate_batch_for_writer(writer_id: i32) -> RecordBatch {
    let schema = Arc::new(ArrowSchema::new(vec![
        Field::new("int", ArrowDataType::Int32, false),
        Field::new("string", ArrowDataType::Utf8, true),
        Field::new(
            "timestamp",
            ArrowDataType::Timestamp(TimeUnit::Microsecond, None),
            true,
        ),
        Field::new("writer_id", ArrowDataType::Int32, false),
    ]));

    let int_values = Int32Array::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    let str_values = StringArray::from(vec!["A", "B", "A", "B", "A", "A", "A", "B", "B", "A", "A"]);
    let ts_values = TimestampMicrosecondArray::from(vec![
        1000000012, 1000000012, 1000000012, 1000000012, 500012305, 500012305, 500012305, 500012305,
        500012305, 500012305, 500012305,
    ]);
    let writer_id_values = Int32Array::from(vec![writer_id; 11]);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(int_values),
            Arc::new(str_values),
            Arc::new(ts_values),
            Arc::new(writer_id_values),
        ],
    )
    .unwrap()
}

async fn writer_task(table_uri: String, writer_id: i32) {
    let mut attempts = 0;
    loop {
        let ops = DeltaOps::try_from_uri(&table_uri).await.expect("Failed to create ops");
        let (table, _) = match ops.load().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Writer {} failed to load table: {:?}", writer_id, e);
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let batch = generate_batch_for_writer(writer_id);
        let result = DeltaOps(table)
            .write(vec![batch])
            .with_save_mode(SaveMode::Append)
            .await;

        match result {
            Ok(_) => {
                println!("Writer {} succeeded after {} attempts", writer_id, attempts);
                return;
            }
            Err(e) => {
                eprintln!("Writer {} attempt {} failed: {:?}", writer_id, attempts, e);
                attempts += 1;
                if attempts >= 10 {
                    panic!("Writer {} failed after 10 attempts", writer_id);
                }
                tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
            }
        }
    }
}

async fn reader_task(table_uri: String, stop_flag: Arc<AtomicBool>) {
    while !stop_flag.load(Ordering::Relaxed) {
        let ops = DeltaOps::try_from_uri(&table_uri).await.expect("Failed to create ops");
        let (_, stream) = ops.load().await.expect("Failed to load table");
        match collect_sendable_stream(stream).await {
            Ok(data) => {
                let total_rows: usize = data.iter().map(|b| b.num_rows()).sum();
                println!("Read {} records", total_rows);
            }
            Err(e) => eprintln!("Read failed: {:?}", e),
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Read a full snapshot of the table in batch
async fn reader_task_snapshot(table_uri: String) -> HashSet<i32> {
    // Open the table via DeltaTable
    let table = open_table(table_uri).await.unwrap();
    let ctx = SessionContext::new();
    ctx.register_table("my_table", Arc::new(table)).unwrap();

    // Run a simple SELECT against it
    let df = ctx.sql("SELECT * FROM my_table").await.unwrap();
    let results = df.collect().await.unwrap();

    println!("table content: {:?}", results);
    // Gather all writer_ids
    let mut unique_ids = HashSet::new();
    if !results.is_empty() {
        let combined = concat_batches(&results[0].schema(), &results).unwrap();
        let idx = combined
            .schema()
            .index_of("writer_id")
            .expect("writer_id not found");
        
        println!("found idx at : {}", idx);
        let writer_id_array = combined
            .column(idx)
            .as_any()
            .downcast_ref::<Int32Array>()
            .expect("writer_id not found");

        for val in writer_id_array.values() {
            unique_ids.insert(*val);
        }
    }
    unique_ids
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let table_uri = temp_dir.path().to_str().unwrap().to_string();

    // Create the initial table
    let ops = DeltaOps::try_from_uri(&table_uri).await.unwrap();
    ops.create()
        .with_columns(get_test_table_columns())
        .with_partition_columns(["timestamp"])
        .with_table_name("concurrent_test_table")
        .await
        .unwrap();

    let stop_flag = Arc::new(AtomicBool::new(false));

    // Spawn reader tasks
    let reader_stop_flag = stop_flag.clone();
    let reader_table_uri = table_uri.clone();
    tokio::spawn(async move {
        reader_task(reader_table_uri, reader_stop_flag).await;
    });

    // Spawn writer tasks
    let mut handles = vec![];
    for writer_id in 0..10 {
        let uri = table_uri.clone();
        handles.push(tokio::spawn(async move {
            writer_task(uri, writer_id).await;
        }));
    }

    // Wait for all writers to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Signal readers to stop
    stop_flag.store(true, Ordering::Relaxed);

    // Verify final table state using DataFusion
    let found_ids = reader_task_snapshot(table_uri.clone()).await;
    for id in 0..10 {
        assert!(found_ids.contains(&id), "Writer ID {} not found", id);
    }

    // Optional: just ensure the total record count is as expected
    assert_eq!(found_ids.len(), 10);
}