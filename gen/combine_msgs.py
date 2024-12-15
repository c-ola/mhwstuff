import json
import os

def combine_json_files(input_folder, output_file):
    combined_data = {}
    for root, dirs, files in os.walk(input_folder):
        for name in files:
            if name.endswith(".msg.23.json"):
                filepath = os.path.join(root, name)
                try:
                    # Load JSON data
                    with open(filepath, 'r') as f:
                        data = json.load(f)
                        # Merge JSON content
                        combined_data.update(data)
                except Exception as e:
                    print(f"Error processing file {filepath}: {e}")

    # Write combined data to the output file
    with open(output_file, 'w') as f:
        json.dump(combined_data, f, indent=4)

    print(f"Combined JSON saved to {output_file}")

# Example usage
input_folder = "./outputs/stm"  # Replace with your folder path
output_file = "./outputs/combined_msgs.json"   # Replace with your desired output file path
combine_json_files(input_folder, output_file)

