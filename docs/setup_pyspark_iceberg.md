# Setting Up PySpark with Apache Iceberg

This document provides the steps and required JAR files to set up a local PySpark instance to work with Apache Iceberg.

## Docker Compose Setup

Below is the Docker Compose file from Tabular's official documentation to set up a local instance for Apache Iceberg using Spark, MinIO, and REST catalog. You can also check out the link from [tabular.io](https://tabular.io/blog/docker-spark-and-iceberg-the-fastest-way-to-try-iceberg/). 

```yaml
version: "3"

services:
  spark-iceberg:
    image: tabulario/spark-iceberg
    container_name: spark-iceberg
    build: spark/
    networks:
      iceberg_net:
    depends_on:
      - rest
      - minio
    volumes:
      - ./warehouse:/home/iceberg/warehouse
      - ./notebooks:/home/iceberg/notebooks/notebooks
    environment:
      - AWS_ACCESS_KEY_ID=admin
      - AWS_SECRET_ACCESS_KEY=password
      - AWS_REGION=us-east-1
    ports:
      - 8888:8888
      - 8080:8080
      - 10000:10000
      - 10001:10001
  rest:
    image: tabulario/iceberg-rest
    container_name: iceberg-rest
    networks:
      iceberg_net:
    ports:
      - 8181:8181
    environment:
      - AWS_ACCESS_KEY_ID=admin
      - AWS_SECRET_ACCESS_KEY=password
      - AWS_REGION=us-east-1
      - CATALOG_WAREHOUSE=s3://warehouse/
      - CATALOG_IO__IMPL=org.apache.iceberg.aws.s3.S3FileIO
      - CATALOG_S3_ENDPOINT=http://minio:9000
  minio:
    image: minio/minio
    container_name: minio
    environment:
      - MINIO_ROOT_USER=admin
      - MINIO_ROOT_PASSWORD=password
      - MINIO_DOMAIN=minio
    networks:
      iceberg_net:
        aliases:
          - warehouse.minio
    ports:
      - 9001:9001
      - 9000:9000
    command: ["server", "/data", "--console-address", ":9001"]
  mc:
    depends_on:
      - minio
    image: minio/mc
    container_name: mc
    networks:
      iceberg_net:
    environment:
      - AWS_ACCESS_KEY_ID=admin
      - AWS_SECRET_ACCESS_KEY=password
      - AWS_REGION=us-east-1
    entrypoint: >
      /bin/sh -c "
      until (/usr/bin/mc config host add minio http://minio:9000 admin password) do echo '...waiting...' && sleep 1; done;
      /usr/bin/mc rm -r --force minio/warehouse;
      /usr/bin/mc mb minio/warehouse;
      /usr/bin/mc policy set public minio/warehouse;
      tail -f /dev/null
      "      
networks:
  iceberg_net:

```

My own modified version can be found [here](/archive/code_backup/docker-compose.yml).

## Required JAR Files

Below is the list of JAR files that need to be added to your PySpark setup:

| JAR File Name                                |
|----------------------------------------------|
| aws-crt-0.30.5.jar                           |
| aws-java-sdk-core-1.12.767.jar               |
| aws-java-sdk-dynamodb-1.12.767.jar           |
| aws-java-sdk-s3-1.12.767.jar                 |
| hadoop-aws-3.3.4.jar                         |
| iceberg-aws-bundle-1.6.0.jar                 |
| iceberg-spark-extensions-3.5_2.12.jar        |
| s3-transfer-manager-2.26.31.jar              |

All of them can be downloaded from the [Maven Central Repository](https://mvnrepository.com/).

## Steps to Add JAR Files

To successfully add the required JAR files to a containerized PySpark instance, follow these steps:

1. **Download the Required JAR Files**: Download the required JAR files from their respective repositories or Maven Central.

2. **Mount the JAR Files into the Container**: In your `docker-compose.yml` file, mount the folder containing the JAR files into the container. For example:
    ```yaml
    services:
      spark-iceberg:
        ...
        volumes:
          - ./extra-jars:/tmp/extra-jars
        ...
    ```

3. **Copy the JAR Files Inside the Container**: Create a script to copy the JAR files to the Spark JAR folder inside the container. For example, create a script named `copy-jars.sh`:
    ```bash
    #!/bin/bash
    cp /tmp/extra-jars/*.jar /opt/spark/jars/
    ```

4. **Update the Docker Compose File**: Ensure the script is executed when the container starts. You can add this to your Dockerfile or entrypoint script.

5. **Configure PySpark to Include the JAR Files**: When initializing the SparkSession, configure PySpark to include the JAR files. For example:
    ```python
    from pyspark.sql import SparkSession

    spark_jars = """
        /opt/spark/jars/iceberg-spark-runtime-3.5_2.12-1.5.0.jar,
        /opt/spark/jars/iceberg-aws-bundle-1.6.0.jar,
        /opt/spark/jars/aws-java-sdk-s3-1.12.767.jar,
        /opt/spark/jars/aws-java-sdk-core-1.12.767.jar,
        /opt/spark/jars/s3-transfer-manager-2.26.31.jar,
        /opt/spark/jars/aws-crt-0.30.5.jar,
        /opt/spark/jars/hadoop-aws-3.3.4.jar,
        /opt/spark/jars/aws-java-sdk-dynamodb-1.12.767.jar
    """

    spark = SparkSession.builder \
        .appName("IcebergExample") \
        .config("spark.jars", spark_jars) \
        .config("spark.driver.extraClassPath", "/opt/spark/jars/*") \
        .config("spark.executor.extraClassPath", "/opt/spark/jars/*") \
        .getOrCreate()
    ```

By following these steps, you can ensure that all the required JAR files are correctly added to your containerized PySpark instance and are recognized by PySpark.
