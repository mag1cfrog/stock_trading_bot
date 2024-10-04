from dataclasses import dataclass
from enum import Enum
import sys
from pathlib import Path

from loguru import logger

class LogLevel(Enum):
    TRACE = "TRACE"
    DEBUG = "DEBUG"
    INFO = "INFO"
    SUCCESS = "SUCCESS"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"


@dataclass
class LoggerConfig:
    log_file_path: Path
    log_level: LogLevel = LogLevel.DEBUG


def setup_logging(config: LoggerConfig):
    """
    Configures Loguru for logging throughout the application.

    Args:
        config (LoggerConfig): The configuration for the logger.

    Returns:
        logger: The configured logger object.
    
    Examples:
        >>> config = LoggerConfig(log_file_path="logs/app.log", log_level=LogLevel.DEBUG)
        >>> logger = setup_logging(config)
    
    """
    # Remove the default logger to prevent duplicate logs
    logger.remove()
    
    # Add a new logger that outputs to stdout with desired formatting
    logger.add(sys.stdout, format="{time:YYYY-MM-DD HH:mm:ss} | {level} | {name} | {message}", level=config.log_level.value)
    
    # Optionally, add a file logger
    logger.add(config.log_file_path, rotation="10 MB", retention="10 days", level="INFO", format="{time:YYYY-MM-DD HH:mm:ss} | {level} | {name} | {message}")
    
    return logger