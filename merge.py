import os
import re

def merge_files_without_tags(input_folder, output_file):
    # Ensure the output file doesn't exist before starting
    if os.path.exists(output_file):
        os.remove(output_file)

    # Open the output file in write mode
    with open(output_file, "w", encoding="utf-8") as outfile:
        # Iterate through all files in the input folder
        for filename in sorted(os.listdir(input_folder)):
            file_path = os.path.join(input_folder, filename)
            if os.path.isfile(file_path):  # Check if it's a file
                with open(file_path, "r", encoding="utf-8") as infile:
                    for line in infile:
                        # Remove any text enclosed in angle brackets, including the brackets
                        cleaned_line = re.sub(r"<[^>]+>", "", line)
                        # Write the cleaned line to the output file
                        outfile.write(cleaned_line)

    print(f"Merged files into {output_file} without tags.")

# Folder containing the files
input_folder = "../lat_text_tesserae/texts"

# Output file
output_file = "gigafile.txt"

# Run the merging process
merge_files_without_tags(input_folder, output_file)
