use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use deltalake::arrow::{
    array::{Int32Array, StringArray, TimestampMicrosecondArray},
    datatypes::{DataType as ArrowDataType, Field, Schema as ArrowSchema, TimeUnit},
    record_batch::RecordBatch,
};
use deltalake::kernel::{DataType, PrimitiveType, StructField};
use deltalake::operations::collect_sendable_stream;
use deltalake::protocol::SaveMode;
use deltalake::DeltaOps;

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

    // Verify the final table state
    let ops = DeltaOps::try_from_uri(&table_uri).await.unwrap();
    let (_, stream) = ops.load().await.unwrap();
    let data = collect_sendable_stream(stream).await.unwrap();
    let combined = deltalake::arrow::compute::concat_batches(&data[0].schema(), &data).unwrap();

    println!("Final table contents:");
    println!("{:?}", combined);

    // Check all writer_ids are present
    let writer_id_array = combined.column(2).as_any().downcast_ref::<Int32Array>().expect("Writer id array not found");
    let mut unique_writer_ids = HashSet::new();
    for id in writer_id_array.values() {
        unique_writer_ids.insert(*id);
    }

    for id in 0..10 {
        assert!(
            unique_writer_ids.contains(&id),
            "Writer ID {} not found in table",
            id
        );
    }

    assert_eq!(writer_id_array.len(), 110, "Total records mismatch");
}