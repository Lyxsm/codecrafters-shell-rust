#[cfg(test)]
mod tests {
    use crate::cmd::*;
    use std::{
	    path::{Path},
	    env,
    };

    #[test]
    fn test_single_quotes_only() {
        let input = "This is a 'single quote'.";
        find_quotes(input); 
        // Expected output: Should find one single quote at index 15 and 30
    }

    #[test]
    fn test_double_quotes_only() {
        let input = "This is a \"double quote\" test.";
        find_quotes(input); 
        // Expected output: Should find double quotes at index 10 and 26
    }

    #[test]
    fn test_mixed_quotes() {
        let input = "'Single quotes' and \"double quotes\" mixed.";
        find_quotes(input);
        // Expected output: Single quotes between 0-14 and double quotes between 19-35
    }

    #[test]
    fn test_quotes_inside_quotes() {
        let input = "'This \"string\" has nested quotes'.";
        find_quotes(input); 
        // Expected output: Single quotes around the whole string, double quotes inside it.
    }

    #[test]
    fn test_unbalanced_single_quotes() {
        let input = "'This is an unbalanced quote.";
        find_quotes(input); 
        // Expected output: Should find a single quote at index 0 and 28 (open), but no closing quote.
    }

    #[test]
    fn test_unbalanced_double_quotes() {
        let input = "\"This is an unbalanced quote.";
        find_quotes(input); 
        // Expected output: Should find a double quote at index 0 but no closing quote.
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        find_quotes(input);
        // Expected output: No quotes, so no output or changes.
    }

    #[test]
    fn test_no_quotes() {
        let input = "No quotes here!";
        find_quotes(input);
        // Expected output: No quotes found, so no output or changes.
    }

    #[test]
    fn test_quotes_at_beginning_and_end() {
        let input = "'Start and end with quotes'";
        find_quotes(input); 
        // Expected output: Single quotes at index 0 and 27
    }

    #[test]
    fn test_double_quotes_at_end() {
        let input = "Ends with a \"double quote\"";
        find_quotes(input); 
        // Expected output: Double quotes found at indices 14 and 30
    }

    #[test]
    fn test_single_quotes_at_end() {
        let input = "Ends with a 'single quote'";
        find_quotes(input); 
        // Expected output: Single quotes found at indices 13 and 29
    }

    #[test]
    fn test_quotes_inside_string() {
        let input = "The string contains a 'quote' here and \"double quotes\" there.";
        find_quotes(input);
        // Expected output: Single quotes around 'quote', and double quotes around "double quotes"
    }

    #[test]
    fn test_repeated_quotes() {
        let input = "'Repeated' 'quotes' 'in' the 'sentence'.";
        find_quotes(input); 
        // Expected output: Single quotes found around each 'Repeated', 'quotes', 'in', 'sentence'
    }

    #[test]
    fn test_consecutive_quotes() {
        let input = "\"\"\"Triple double quotes\"\"\"";
        find_quotes(input); 
        // Expected output: Triple double quotes found around the text
    }

    #[test]
    fn test_escaped_quotes() {
        let input = "This is an escaped quote: \"\\\"escaped\\\"\"";
        find_quotes(input); 
        // Expected output: This should not remove the escaped quotes and should handle the literal quotes.
        // NOTE: This case will fail because current code doesn't handle escaped quotes.
    }

    #[test]
    fn test_quotes_with_spaces() {
        let input = "This 'is' a 'quote' with spaces inside.";
        find_quotes(input); 
        // Expected output: Should identify single quotes around 'is' and 'quote'
    }

    #[test]
    fn test_nested_quotes_with_spaces() {
        let input = "'This is a \"nested quote\" inside'.";
        find_quotes(input); 
        // Expected output: Single quotes around the whole string, double quotes around "nested quote".
    }

    #[test]
    fn test_quotes_with_newlines() {
        let input = "'Single quote\nTest' and \"double\nquote\" here.";
        find_quotes(input); 
        // Expected output: Should handle the newline within quotes properly.
    }

    #[test]
    fn test_multiple_single_quotes() {
        let input = "Here is 'one' and 'two' single quotes.";
        find_quotes(input); 
        // Expected output: Single quotes around 'one' and 'two'
    }

    #[test]
    fn test_multiple_double_quotes() {
        let input = "Here are \"three\" and \"four\" double quotes.";
        find_quotes(input); 
        // Expected output: Double quotes around "three" and "four"
    }

    #[test]
    fn test_combined_multiple_quotes() {
        let input = "'First single' \"second double\" 'third single'.";
        find_quotes(input); 
        // Expected output: Should find single quotes around 'First single' and 'third single' and double quotes around "second double"
    }

    #[test]
    fn test_single_empty_quote() {
        let input = "An empty quote: ''";
        find_quotes(input); 
        // Expected output: Single quotes around the empty string ''
    }

    #[test]
    fn test_double_empty_quote() {
        let input = "An empty quote: \"\"";
        find_quotes(input); 
        // Expected output: Double quotes around the empty string ""
    }

    #[test]
    fn test_parse_with_builtin_command() {
        let input = "echo Hello, World!";
        let (cmd_type, cmd, args) = parse(input);
        assert_eq!(cmd_type, Type::BuiltIn);
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["Hello,".to_string(), "World!".to_string()]);
    }

    #[test]
    fn test_parse_with_invalid_command() {
        let input = "invalid_command";
        let (cmd_type, cmd, args) = parse(input);
        assert_eq!(cmd_type, Type::Invalid);
        assert_eq!(cmd, "invalid_command");
        assert!(args.is_empty());
    }

    #[test]
    fn test_cmd_split() {
        let input = "echo Hello, World!";
        let (cmd, args) = cmd_split(input);
        assert_eq!(cmd, "echo");
        assert_eq!(args, "Hello, World!");
    }

    #[test]
    fn test_is_executable_with_valid_file() {
        let path = Path::new("/bin/ls");
        assert!(is_executable(path));
    }

    #[test]
    fn test_is_executable_with_invalid_file() {
        let path = Path::new("/bin/nonexistent");
        assert!(!is_executable(path));
    }

    #[test]
    fn test_cmd_type_with_builtin() {
        let cmd = "echo";
        assert_eq!(cmd_type(cmd), Type::BuiltIn);
    }

    #[test]
    fn test_cmd_type_with_path_exec() {
        let cmd = "ls";
        assert_eq!(cmd_type(cmd), Type::PathExec);
    }

    #[test]
    fn test_cmd_type_with_invalid() {
        let cmd = "nonexistent_command";
        assert_eq!(cmd_type(cmd), Type::Invalid);
    }

    #[test]
    fn test_find_in_path_with_existing_binary() {
        let binary = "ls";
        assert!(find_in_path(binary).is_some());
    }

    #[test]
    fn test_find_in_path_with_nonexistent_binary() {
        let binary = "nonexistent_binary";
        assert!(find_in_path(binary).is_none());
    }

    #[test]
    fn test_get_path_entries() {
        let entries = get_path_entries();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_change_dir_to_home() {
        change_dir("~");
        let current_dir = env::current_dir().unwrap();
        assert_eq!(current_dir, env::home_dir().unwrap());
    }

    #[test]
    fn test_change_dir_to_invalid_path() {
        change_dir("/nonexistent_path");
        let current_dir = env::current_dir().unwrap();
        assert_ne!(current_dir, Path::new("/nonexistent_path"));
    }

    #[test]
    fn test_parse_args_with_quotes() {
        let input = "echo \"Hello, World!\"";
        let args = parse_args(input);
        assert_eq!(args, vec!["echo".to_string(), "Hello, World!".to_string()]);
    }

    #[test]
    fn test_parse_args_with_multiple_spaces() {
        let input = "echo    Hello,    World!";
        let args = parse_args(input);
        assert_eq!(args, vec!["echo".to_string(), "Hello,".to_string(), "World!".to_string()]);
    }

    #[test]
    fn test_find_quotes_with_unbalanced_quotes() {
        let input = "'Unbalanced quote";
        let quotes = find_quotes(input);
        assert!(quotes.is_empty());
    }
}