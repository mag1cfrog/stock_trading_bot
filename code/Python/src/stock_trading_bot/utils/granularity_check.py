import polars as pl

def is_base_granularity(data: pl.DataFrame, base_amount: int, base_unit: str) -> bool:
    """
    Check if the data is at the base granularity level.
    """
    # Calculate the expected difference in seconds based on the base time unit and amount
    if base_unit == "seconds":
        expected_diff_seconds = base_amount
    elif base_unit == "minutes":
        expected_diff_seconds = base_amount * 60
    elif base_unit == "hours":
        expected_diff_seconds = base_amount * 3600
    elif base_unit == "days":
        expected_diff_seconds = base_amount * 86400
    else:
        raise ValueError("Unsupported time unit for base granularity")

    # Convert timestamp differences to seconds
    time_diffs = data.with_columns(
        pl.col('timestamp').diff().fill_none(0).cast(pl.Int64).alias('diff')
    )

    # Assuming stock market hours are from 9:30 AM to 4:00 PM local time
    normal_trading_hours = data.filter(
        (pl.col('timestamp').dt.hour() * 60 + pl.col('timestamp').dt.minute() >= 570) &  # Market opens at 9:30 AM
        (pl.col('timestamp').dt.hour() * 60 + pl.col('timestamp').dt.minute() <= 960)    # Market closes at 4:00 PM
    )

    # Filter time differences to only those during normal trading hours
    filtered_diffs = normal_trading_hours.join(
        time_diffs,
        how='left',
        on='timestamp'
    )['diff']

    # Define a tolerance for time difference (e.g., a few seconds)
    tolerance = 10  # seconds

    # Check if all time differences are within the expected range considering the tolerance
    return all((filtered_diffs.abs() - expected_diff_seconds).abs() < tolerance)
