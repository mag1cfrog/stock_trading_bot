import os


def get_unique_highest_title(file_path):
    highest_level = 7  # More than the maximum markdown levels
    title = None

    with open(file_path, "r", encoding="utf-8") as file:
        for line in file:
            # Check if the line starts with a Markdown header
            if line.startswith("#"):
                level = line.count("#")

                # Only update if this is a header line and a higher level (lower number) than previously found
                if 0 < level < highest_level:
                    # Reset when a new higher level is found
                    highest_level = level
                    title = line.strip("# \n")
                    count = 1
                elif level == highest_level:
                    # Count occurrences of this level
                    count += 1

    # Only return the title if it's uniquely at the highest level
    if count == 1:
        return title
    return None


# Path to the dev-log directory
dev_log_dir = "./docs/dev-log"
index_file_path = os.path.join(dev_log_dir, "index.md")

# Fetch all markdown files
log_files = [
    f for f in os.listdir(dev_log_dir) if f.endswith(".md") and f != "index.md"
]
log_files.sort()

# Create or overwrite the index file
with open(index_file_path, "w") as index_file:
    index_file.write("# Development Log Index\n\n")
    for log_file in log_files:
        file_path = os.path.join(dev_log_dir, log_file)
        date = log_file.split(".")[0]
        title = f"Development Log Entry - {date}"
        unique_title = get_unique_highest_title(file_path)
        entry = f"- [{title}]({log_file})"
        # If a unique highest-level title is found, append it to the entry
        if unique_title:
            entry += f" - {unique_title}"
        index_file.write(entry + "\n")

print("Index file has been created successfully.")
