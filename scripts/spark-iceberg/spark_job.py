from pyspark.sql import SparkSession

spark = (
    SparkSession.builder.appName("testing")
    .master("local")
    .config("spark.sql.defaultCatalog", "demo")
    .config("spark.driver.extraClassPath", "/opt/spark/jars/*")
    .config("spark.executor.extraClassPath", "/opt/spark/jars/*")
    .config(
        "spark.jars",
        """
                /opt/spark/jars/iceberg-spark-runtime-3.5_2.12-1.5.0.jar, 
                /opt/spark/jars/iceberg-aws-bundle-1.6.0.jar,
                /opt/spark/jars/aws-java-sdk-s3-1.12.767.jar,
                /opt/spark/jars/aws-java-sdk-core-1.12.767.jar,
                /opt/spark/jars/s3-transfer-manager-2.26.31.jar,
                /opt/spark/jars/aws-crt-0.30.5.jar,
                /opt/spark/jars/hadoop-aws-3.3.4.jar,
                /opt/spark/jars/aws-java-sdk-dynamodb-1.12.767.jar
                """,
    )
    .config(
        "spark.sql.extensions",
        "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions",
    )
    .config("spark.sql.catalog.demo", "org.apache.iceberg.spark.SparkCatalog")
    .config("spark.sql.catalog.demo.type", "rest")
    .config("spark.sql.catalog.demo.uri", "http://rest:8181")
    .config("spark.sql.catalog.demo.io-impl", "org.apache.iceberg.aws.s3.S3FileIO")
    .config("spark.sql.catalog.demo.warehouse", "s3a://warehouse")
    .config("spark.hadoop.fs.s3a.endpoint", "http://minio:9000")
    .config("spark.hadoop.fs.s3a.access.key", "admin")
    .config("spark.hadoop.fs.s3a.secret.key", "password")
    .config("spark.hadoop.fs.s3a.path.style.access", "true")
    .config("spark.hadoop.fs.s3a.impl", "org.apache.hadoop.fs.s3a.S3AFileSystem")
    .enableHiveSupport()
    .getOrCreate()
)
df = spark.createDataFrame([(1,), (3,), (5,)], ["id"])

df.write.format("iceberg").mode("overwrite").saveAsTable("demo.default.test_table")


spark.stop()
