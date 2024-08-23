from pyiceberg.catalog.rest import RestCatalog
from requests import Session

# Configuration properties
properties = {
    'uri': 'http://localhost:8181',
    'warehouse_location': 's3a://warehouse/wh',
    'file_io_impl': 'org.apache.iceberg.aws.s3.S3FileIO',
    'endpoint': 'http://localhost:9000'
}

# Initialize the REST catalog
catalog = RestCatalog(name="demo", **properties)

# Create and configure a session
session = Session()
session.verify = False  # Turn off SSL verification

# Use the session to interact with your services
response = session.get('http://localhost:8181/namespaces')
print(response.text)

from pyiceberg.schema import Schema
from pyiceberg.types import StringType, TimestamptzType, ListType, NestedField

# Define the schema using NestedField for schema fields
schema = Schema(
    fields=[
        NestedField(field_id=1, name="level", field_type=StringType(), required=True),
        NestedField(field_id=2, name="event_time", field_type=TimestamptzType(), required=True),  # Using TimestamptzType for timezone
        NestedField(field_id=3, name="message", field_type=StringType(), required=True),
        NestedField(field_id=4, name="call_stack", field_type=ListType(element_id=5, element_type=StringType(), element_required=True), required=True)
    ]
)

from pyiceberg.partitioning import PartitionSpec, PartitionField
from pyiceberg.transforms import YearTransform, MonthTransform, IdentityTransform

# Assuming 'schema' is already defined and includes a timestamp field for event times
partition_spec = PartitionSpec(
    PartitionField(field_id=1, source_id=schema.find_field("event_time").field_id, name="event_time_year", transform=YearTransform()),
    PartitionField(field_id=2, source_id=schema.find_field("event_time").field_id, name="event_time_month", transform=MonthTransform()),
    PartitionField(field_id=3, source_id=schema.find_field("level").field_id, name="level_identity", transform=IdentityTransform())
)


from pyiceberg.catalog import load_catalog

# Load your catalog with the properties
catalog = load_catalog('demo', **properties)

# Define your namespace and table
namespace = "test"
table_name = "logs"
full_table_name = f"{namespace}.{table_name}"

# Attempt to create the table
catalog.create_table(full_table_name, schema, location=None, partition_spec=partition_spec)


# Now you can use the catalog to perform operations such as listing tables, loading tables, etc.
# For example, listing all tables in the namespace:
tables = catalog.list_tables(namespace='test')
print(tables)

# Load a specific table
table = catalog.load_table(identifier='test.logs')
print(table)