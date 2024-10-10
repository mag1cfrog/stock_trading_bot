import polars as pl


def base_granularity_health_score(
    data: pl.DataFrame, base_unit: str, base_amount: int
) -> float:
    """
    Calculate the health score of the data based on the base granularity.

    Args:
    data: The input data with a timestamp column.
    base_unit: The base unit of the granularity (e.g., "minutes").
    base_amount: The base amount of the granularity (e.g., 5).

    Returns:
    The health score of the data based on the base granularity.

    Raises:
    ValueError: If the base unit is not supported.

    Example:
    data = pl.DataFrame({
        'timestamp': ['2021-01-01 00:00:00', '2021-01-01 00:05:00', '2021-01-01 00:10:00', '2021-01-01 00:20:00']
    })
    base_granularity_health_score(data, "minutes", 5)
    # Output
    100.0


    """

    # Calculate the expected difference in seconds based on the base time unit and amount
    unit_to_seconds = {"seconds": 1, "minutes": 60, "hours": 3600, "days": 86400}
    if base_unit not in unit_to_seconds:
        raise ValueError("Unsupported time unit for base granularity")

    expected_diff_seconds = base_amount * unit_to_seconds[base_unit]

    # Convert timestamps to a date format (yyyy-mm-dd)
    data_with_date = data.with_columns(pl.col("timestamp").dt.date().alias("date"))

    # Group data by date and check intervals within each date
    grouped_data = data_with_date.group_by("date")

    total_expected_count = 0
    total_actual_fit_count = 0

    # Calculate expected and actual fitting records for each group
    for _, group in grouped_data:
        max_timestamp = group.select(pl.max("timestamp")).to_struct()[0]["timestamp"]
        min_timestamp = group.select(pl.min("timestamp")).to_struct()[0]["timestamp"]
        day_range_seconds = (max_timestamp - min_timestamp).total_seconds()
        expected_count = day_range_seconds // expected_diff_seconds

        # Get the actual records for that day
        actual_data = group.sort("timestamp")
        actual_diffs = actual_data.with_columns(
            pl.col("timestamp")
            .dt.epoch(time_unit="s")
            .diff()
            .cast(pl.Int64)
            .alias("diff")
        ).drop_nulls()["diff"]

        # Calculate the number of records fitting the expected interval
        fit_count = (actual_diffs == expected_diff_seconds).sum()

        total_expected_count += expected_count
        total_actual_fit_count += fit_count

    # Calculate overall health percentage
    if total_expected_count > 0:
        health_percentage = (total_actual_fit_count / total_expected_count) * 100
    else:
        health_percentage = (
            100  # No expected data (e.g., non-trading days), assume perfect health
        )

    return health_percentage
