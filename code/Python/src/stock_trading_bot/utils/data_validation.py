from loguru import logger
import pandera.polars as pa
import polars as pl


def validate_pl_df(data: pl.DataFrame):
    """
    Validate a Polars DataFrame against a schema, also checking for non-negative values.
    """
    non_negative_checks = [
        pa.Check.greater_than_or_equal_to(0)
    ]
    schema = pa.DataFrameSchema(
        columns={
            'symbol': pa.Column(pa.String),
            'timestamp': pa.Column(pl.Datetime(time_unit='ns', time_zone='UTC')),
            'open': pa.Column(pa.Float, checks=non_negative_checks),
            'high': pa.Column(pa.Float, checks=non_negative_checks),
            'low': pa.Column(pa.Float, checks=non_negative_checks),
            'close': pa.Column(pa.Float, checks=non_negative_checks),
            'volume': pa.Column(pa.Int, checks=non_negative_checks, coerce=True),
            'trade_count': pa.Column(pa.Int, checks=non_negative_checks, coerce=True),
            'vwap': pa.Column(pa.Float, checks=non_negative_checks),
        }
    )

    logger.trace(f"Validating DataFrame schema")
    logger.exception(schema.validate(data))