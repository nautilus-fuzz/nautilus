import json
import re
grammar_name = 'Lua'
def convert_json_to_antlr(json_grammar):
    antlr_grammar = []

    # The first key in the JSON is considered the start rule.
    start_rule = list(json_grammar.keys())[0]
    
    # Start with the grammar declaration.
    antlr_grammar.append(f"grammar {grammar_name};\n")

    # Iterate over each non-terminal in the JSON grammar.
    for non_terminal, productions in json_grammar.items():
        # Convert JSON non-terminal to ANTLR rule name.
        rule_name = non_terminal.replace("<", "").replace(">", "")
        antlr_rule = f"{rule_name}\n    : "

        # Convert each production for the current non-terminal.
        production_strings = []
        for production in productions:
            # Convert each symbol in the production to its ANTLR form.
            production_string = " ".join(
                symbol.replace("<", "").replace(">", "") if symbol.startswith("<") and symbol.endswith(">") else f"'{symbol}'"
                for symbol in production
            )
            # Ensure escape sequences are written as literals by escaping backslashes.
            production_strings.append(production_string.replace("\\", "\\\\") if production_string else "/* empty */")

        # Join all productions with the '|' operator.
        antlr_rule += "\n    | ".join(production_strings) + "\n    ;\n"
        antlr_grammar.append(antlr_rule)

    full_grammar = "\n".join(antlr_grammar)
    # Remove exessive slashed
    full_grammar = full_grammar.replace("\\\\", "\\")
    # escape ' instead of "
    full_grammar = full_grammar.replace("\\\"", "\"")
    full_grammar = full_grammar.replace("'''", "'\\''")

    #full_grammar = remove_whitespace_and_newlines(full_grammar)
    full_grammar = remove_whitespace_within_quotes(full_grammar)

    # to lowecase
    full_grammar = replace_uppercase_substrings(full_grammar)
    full_grammar = full_grammar + "\nWHITESPACE\n    : " + "(' ' | '\\n' | '\\t' | '\\r')+ -> skip" + "\n    ;\n"
    # Join all the rules into the final ANTLR grammar string.
    return full_grammar

def load_json_grammar(file_path):
    # Read the file as text to preserve all escape sequences.
    with open(file_path, 'r') as file:
        raw_content = file.read()
    
    # Use regular expressions to handle escape sequences properly.
    # Replace \\ with \\\\ to double escape backslashes for ANTLR.
    raw_content = raw_content.replace("\\", "\\\\")
    # Escape double quote symbol, or else it breaks json
    raw_content = raw_content.replace("\\\\\"\"", "\\\\\\\"\"")
    #print(raw_content)
    # Convert the raw string to a JSON object.
    json_grammar = json.loads(raw_content)
    
    return json_grammar

def remove_whitespace_and_newlines(s):
    s = s.replace('\n', '')
    s = s.replace('\r', '')
    s = s.replace('\t', '')
    s = s.replace(' ', '')
    return s

def remove_whitespace_within_quotes(s):
    # Define a regular expression pattern to match text within single quotes
    pattern = re.compile(r"'(.*?)'")

    # Function to remove whitespace characters from a matched group
    def remove_whitespace(match):
        # Get the content inside quotes
        content = match.group(1)
        # Remove all whitespace characters
        cleaned_content = re.sub(r' |(?<!\\)\\n', '', content)
        #cleaned_content = re.sub(r'\n', '', content)
        # Return the cleaned content enclosed in single quotes
        return f"'{cleaned_content}'"

    # Use re.sub with the pattern and the cleaning function
    result = pattern.sub(remove_whitespace, s)
    return result

def replace_uppercase_substrings(text):
    # Define a function that will convert matched groups to lowercase
    def to_lowercase(match):
        return match.group(0).lower()
    
    # Use regular expressions to find substrings with at least 3 uppercase letters
    result = re.sub(r'[A-Z]{3,}', to_lowercase, text)
    
    return result

def convert_grammar_files(json_input_file, antlr_output_file):
    # Load the JSON grammar from the input file, preserving escape sequences.
    json_grammar = load_json_grammar(json_input_file)
    
    # Convert the JSON grammar to ANTLR format.
    antlr_grammar = convert_json_to_antlr(json_grammar)
    
    # Write the ANTLR grammar to the output file.
    with open(antlr_output_file, 'w') as file:
        file.write(antlr_grammar)

# Example usage
json_input_file = 'ruby-custom.json'  # Path to the input JSON file
antlr_output_file = 'ruby.g4'  # Path to the output ANTLR file

convert_grammar_files(json_input_file, antlr_output_file)

