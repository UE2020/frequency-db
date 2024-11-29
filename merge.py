import os

def combine_text_files(input_folder, output_file, separator="--- File Separator ---\n"):
    """
    Combines all text files in the input_folder into one file.

    Args:
        input_folder (str): Path to the folder containing text files.
        output_file (str): Path to the output file.
        separator (str): A string to separate the contents of each file.
    """
    # Get all text files in the folder
    text_files = [f for f in os.listdir(input_folder) if f.endswith(".txt")]
    text_files.sort()  # Sort files alphabetically
    
    with open(output_file, "w", encoding="utf-8") as outfile:
        for file_name in text_files:
            file_path = os.path.join(input_folder, file_name)
            
            try:
                with open(file_path, "r", encoding="utf-8") as infile:
                    outfile.write(f"--- {file_name} ---\n")  # Add filename as a header
                    outfile.write(infile.read())
                    outfile.write(f"\n{separator}\n")
                    
            except Exception as e:
                print(f"Could not process file {file_name}: {e}")

# Usage
input_folder = "../lat_text_latin_library"
output_file = "gigafile.txt"

combine_text_files(input_folder, output_file)
