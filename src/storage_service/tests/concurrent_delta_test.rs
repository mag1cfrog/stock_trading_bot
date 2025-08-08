use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use deltalake::arrow::{
    array::{Int32Array, StringArray, TimestampMicrosecondArray},
    datatypes::{DataType as ArrowDataType, Field, Schema as ArrowSchema, TimeUnit},
    record_batch::RecordBatch,
};
use deltalake::datafusion::prelude::SessionContext;
use deltalake::kernel::{DataType, PrimitiveType, StructField};
use deltalake::{DeltaOps, open_table};
use rand::Rng;
use tokio::time::sleep;

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
        let batch = generate_batch_for_writer(writer_id);
        let ops = DeltaOps::try_from_uri(&table_uri)
            .await
            .expect("Failed to create ops");

        match ops
            .write(vec![batch])
            .with_save_mode(deltalake::protocol::SaveMode::Append)
            .await
        {
            Ok(_) => {
                println!("Writer {} succeeded after {} attempts", writer_id, attempts);
                return;
            }
            Err(e) => {
                eprintln!(
                    "Writer {} attempted {} failed: {:?}",
                    writer_id, attempts, e
                );
                attempts += 1;
                if attempts >= 10 {
                    panic!("Writer {} failed after 10 attempts", writer_id);
                }
                sleep(Duration::from_millis(100 * attempts)).await
            }
        }
    }
}

/// Read a full snapshot of the table in batch
async fn snapshot_reader_task(table_uri: String, read_counter: Arc<AtomicU32>) -> HashSet<i32> {
    // Open the table via DeltaTable
    let table = open_table(table_uri).await.unwrap();
    let ctx = SessionContext::new();
    ctx.register_table("snapshot_table", Arc::new(table))
        .unwrap();

    // Add random dely to increase concurrent variabiltiy
    let jitter = rand::rng().random_range(50..500);
    sleep(Duration::from_millis(jitter)).await;

    match ctx
        .sql("SELECT DISTINCT writer_id FROM snapshot_table")
        .await
    {
        Ok(df) => {
            let results = df.collect().await.unwrap();
            read_counter.fetch_add(1, Ordering::Relaxed);

            let mut ids = HashSet::new();
            for batch in results {
                let writer_id_array = batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .unwrap();
                for id in writer_id_array.values() {
                    ids.insert(*id);
                }
            }

            ids
        }
        Err(e) => {
            eprintln!("Read failed: {:?}", e);
            HashSet::new()
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_delta_concurrent_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let table_uri = temp_dir.path().to_str().unwrap().to_string();

    // Initialize table
    DeltaOps::try_from_uri(&table_uri)
        .await
        .unwrap()
        .create()
        .with_columns(get_test_table_columns())
        .with_partition_columns(["timestamp"])
        .with_table_name("concurrent_test_table")
        .await
        .unwrap();

    let read_counter = Arc::new(AtomicU32::new(0));

    // Spawn readers
    let mut read_handles = vec![];
    for _ in 0..20 {
        let uri = table_uri.clone();
        let counter = read_counter.clone();
        read_handles.push(tokio::spawn(async move {
            snapshot_reader_task(uri, counter).await
        }));
    }

    // Spawn writer tasks
    let mut writer_handles = vec![];
    for writer_id in 0..10 {
        let uri = table_uri.clone();
        writer_handles.push(tokio::spawn(async move {
            writer_task(uri, writer_id).await;
        }));
    }

    // Wait for all operations to complete
    let (_writer_results, reader_results) = tokio::join!(
        futures::future::join_all(writer_handles),
        futures::future::join_all(read_handles)
    );

    // Verify intermediate reads
    let mut all_read_ids = HashSet::new();
    for result in reader_results {
        let ids = result.unwrap();
        all_read_ids.extend(ids);
    }

    // Verify final state
    let ctx = SessionContext::new();
    let final_table = open_table(&table_uri).await.unwrap();
    ctx.register_table("final_table", Arc::new(final_table))
        .unwrap();

    let final_df = ctx
        .sql("SELECT DISTINCT writer_id FROM final_table")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    let mut final_ids = HashSet::new();
    for batch in final_df {
        let writer_id_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();

        for id in writer_id_array.values() {
            final_ids.insert(*id);
        }
    }

    // Validation
    for id in 0..10 {
        assert!(final_ids.contains(&id), "Missing writer ID {}", id);
    }

    assert_eq!(final_ids.len(), 10);
    println!(
        "Completed {} reads during writes",
        read_counter.load(Ordering::Relaxed)
    );
}
