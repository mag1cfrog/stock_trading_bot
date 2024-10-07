import json
from typing import Dict
import os
from pathlib import Path

from dotenv import load_dotenv
from loguru import logger
import yaml


def load_config_yaml(config_path: str) -> Dict:
    """
    Load a yaml config file into a dictionary

    Args:
        config_path (str): Path to the config file
    Returns:
        Dict: The config file as a dictionary

    
    """
    with open(config_path, 'r') as config_file:
        config = yaml.safe_load(config_file)
    return config


def load_config_json(config_path: str) -> Dict:
    """
    Load a json config file into a dictionary

    Args:
        config_path (str): Path to the config file
    Returns:
        Dict: The config file as a dictionary

    
    """


    with open(config_path, 'r') as config_file:
        config = json.load(config_file)
    return config


def load_config_auto(config_path: Path = None) -> dict:
    """
    Dynamically loads configuration based on the presence of a .env file or an environment variable.
    Can optionally specify a configuration path.
    
    Args:
        config_path (Path, optional): Path to the configuration file. Defaults to None.
        
    Returns:
        dict: The configuration dictionary loaded from a JSON or YAML file.

    Raises:
        FileNotFoundError: If no suitable configuration path is found.
    """
    if config_path:
        return load_config(config_path)

    env_path = find_env_file()
    if env_path:
        logger.debug(f"Loading environment variables from {env_path}")
        load_dotenv(dotenv_path=env_path)
    else:
        logger.error("No .env file found and CONFIG_PATH not set in environment variables.")
        raise FileNotFoundError("No configuration path provided and no .env file found.")

    config_path = os.getenv('CONFIG_PATH')
    if not config_path:
        logger.error("CONFIG_PATH not set in environment variables.")
        raise FileNotFoundError("CONFIG_PATH environment variable is not set.")
    
    logger.debug(f"Loading configuration from {config_path}")
    return load_config(config_path)


def find_env_file() -> str:
    """
    Finds the .env file in the current directory, 'code/Python/' subdirectory,
    or two levels up if the current directory ends with 'code/Python'.
    
    Returns:
        str: The path to the .env file if found, otherwise None.
    """
    current_dir = os.getcwd()
    potential_paths = [
        os.path.join(current_dir, '.env'),
        os.path.join(current_dir, 'code', 'Python', '.env')
    ]

    if current_dir.endswith(os.path.join('code', 'Python')):
        potential_paths.append(os.path.abspath(os.path.join(current_dir, '..', '..', '.env')))

    for path in potential_paths:
        if os.path.exists(path):
            return path

    return None



def load_config(config_path: str) -> dict:
    """
    Load configuration from a JSON or YAML file based on file extension.

    Args:
        config_path (str): Path to the config file

    Returns:
        dict: Configuration dictionary loaded from the file.
    """
    if config_path.endswith('.yaml') or config_path.endswith('.yml'):
        return load_config_yaml(config_path)
    elif config_path.endswith('.json'):
        return load_config_json(config_path)
    else:
        logger.error("Unsupported file format for configuration.")
        raise ValueError("Unsupported file format. Please use either JSON or YAML.")



def main():
    config = load_config_auto()
    print(config)


if __name__ == '__main__':
    main()