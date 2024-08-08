#!/bin/bash
cp /tmp/extra-jars/*.jar /opt/spark/jars/

# Ensure the log directory exists and has proper permissions
mkdir -p /opt/spark/logs
touch /opt/spark/logs/spark.log

# List JARs to verify they are in the correct directory
echo "Listing JAR files in /opt/spark/jars/"
ls -l /opt/spark/jars/

# Start a Spark session that does not exit, here using spark-shell as an example
# Change to your specific Spark application or session command as needed
/opt/spark/bin/spark-shell --conf "spark.ui.showConsoleProgress=true" < /dev/null

# Optionally, you can still tail the log file if the above command ever exits
tail -f /opt/spark/logs/spark.log