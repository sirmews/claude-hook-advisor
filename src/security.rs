//! Default security patterns for detecting risky code

use crate::types::SecurityPattern;

/// Returns all built-in security patterns.
///
/// These patterns are enabled by default and can be disabled via configuration.
pub fn get_default_security_patterns() -> Vec<SecurityPattern> {
    vec![
        // ===== GitHub Actions =====
        SecurityPattern {
            rule_name: "github_actions_workflow".to_string(),
            path_pattern: Some(".github/workflows/*.yml".to_string()),
            content_substrings: vec![],
            reminder: r#"⚠️ Security Warning: You are editing a GitHub Actions workflow file.

Be aware of these security risks:
1. **Command Injection**: Never use untrusted input (issue titles, PR descriptions, commit messages) directly in run: commands
2. **Use environment variables**: Instead of ${{ github.event.issue.title }}, use env: with proper quoting

Example of UNSAFE pattern:
  run: echo "${{ github.event.issue.title }}"

Example of SAFE pattern:
  env:
    TITLE: ${{ github.event.issue.title }}
  run: echo "$TITLE"

Risky inputs: github.event.issue.*, github.event.pull_request.*, github.event.comment.*, github.head_ref"#.to_string(),
        },
        SecurityPattern {
            rule_name: "github_actions_workflow_yaml".to_string(),
            path_pattern: Some(".github/workflows/*.yaml".to_string()),
            content_substrings: vec![],
            reminder: r#"⚠️ Security Warning: You are editing a GitHub Actions workflow file.

Be aware of these security risks:
1. **Command Injection**: Never use untrusted input (issue titles, PR descriptions, commit messages) directly in run: commands
2. **Use environment variables**: Instead of ${{ github.event.issue.title }}, use env: with proper quoting

Example of UNSAFE pattern:
  run: echo "${{ github.event.issue.title }}"

Example of SAFE pattern:
  env:
    TITLE: ${{ github.event.issue.title }}
  run: echo "$TITLE"

Risky inputs: github.event.issue.*, github.event.pull_request.*, github.event.comment.*, github.head_ref"#.to_string(),
        },

        // ===== JavaScript / TypeScript =====
        SecurityPattern {
            rule_name: "eval_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["eval(".to_string()],
            reminder: r#"⚠️ Security Warning: eval() executes arbitrary code and is a major security risk.

Consider using JSON.parse() for data parsing or alternative design patterns that don't require code evaluation.
Only use eval() if you truly need to evaluate arbitrary code."#.to_string(),
        },
        SecurityPattern {
            rule_name: "new_function_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["new Function".to_string()],
            reminder: r#"⚠️ Security Warning: Using new Function() with dynamic strings can lead to code injection vulnerabilities.

Consider alternative approaches that don't evaluate arbitrary code.
Only use new Function() if you truly need to evaluate arbitrary dynamic code."#.to_string(),
        },
        SecurityPattern {
            rule_name: "react_dangerously_set_html".to_string(),
            path_pattern: None,
            content_substrings: vec!["dangerouslySetInnerHTML".to_string()],
            reminder: r#"⚠️ Security Warning: dangerouslySetInnerHTML can lead to XSS vulnerabilities if used with untrusted content.

Ensure all content is properly sanitized using an HTML sanitizer library like DOMPurify, or use safe alternatives."#.to_string(),
        },
        SecurityPattern {
            rule_name: "document_write_xss".to_string(),
            path_pattern: None,
            content_substrings: vec!["document.write".to_string()],
            reminder: r#"⚠️ Security Warning: document.write() can be exploited for XSS attacks and has performance issues.

Use DOM manipulation methods like createElement() and appendChild() instead."#.to_string(),
        },
        SecurityPattern {
            rule_name: "innerHTML_xss".to_string(),
            path_pattern: None,
            content_substrings: vec![".innerHTML =".to_string(), ".innerHTML=".to_string()],
            reminder: r#"⚠️ Security Warning: Setting innerHTML with untrusted content can lead to XSS vulnerabilities.

Use textContent for plain text or safe DOM methods for HTML content.
If you need HTML support, consider using an HTML sanitizer library such as DOMPurify."#.to_string(),
        },
        SecurityPattern {
            rule_name: "child_process_exec".to_string(),
            path_pattern: None,
            content_substrings: vec!["child_process.exec".to_string(), "exec(".to_string(), "execSync(".to_string()],
            reminder: r#"⚠️ Security Warning: Using child_process.exec() can lead to command injection vulnerabilities.

Instead of:
  exec(`command ${userInput}`)

Use:
  import { execFile } from 'child_process'
  execFile('command', [userInput])

The execFile function:
- Uses execFile instead of exec (prevents shell injection)
- Handles arguments as an array (safer than string interpolation)
- Provides proper error handling

Only use exec() if you absolutely need shell features and the input is guaranteed to be safe."#.to_string(),
        },

        // ===== Python =====
        SecurityPattern {
            rule_name: "pickle_deserialization".to_string(),
            path_pattern: None,
            content_substrings: vec!["pickle.loads".to_string(), "pickle.load".to_string()],
            reminder: r#"⚠️ Security Warning: Using pickle with untrusted content can lead to arbitrary code execution.

Consider using JSON or other safe serialization formats instead.
Only use pickle if it is explicitly needed or requested by the user."#.to_string(),
        },
        SecurityPattern {
            rule_name: "os_system_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["os.system(".to_string(), "from os import system".to_string()],
            reminder: r#"⚠️ Security Warning: os.system() can lead to command injection vulnerabilities.

Use subprocess.run() with a list of arguments instead:
  subprocess.run(['command', arg1, arg2])

This should only be used with static arguments and never with arguments that could be user-controlled."#.to_string(),
        },
        SecurityPattern {
            rule_name: "python_eval".to_string(),
            path_pattern: None,
            content_substrings: vec!["eval(".to_string()],
            reminder: r#"⚠️ Security Warning: eval() executes arbitrary Python code and is extremely dangerous.

Never use eval() with user input. Consider alternatives:
- For data: Use ast.literal_eval() or json.loads()
- For expressions: Use a safe expression evaluator
- For dynamic code: Refactor to use functions/methods

eval() is almost never necessary and creates severe security risks."#.to_string(),
        },
        SecurityPattern {
            rule_name: "python_exec".to_string(),
            path_pattern: None,
            content_substrings: vec!["exec(".to_string()],
            reminder: r#"⚠️ Security Warning: exec() executes arbitrary Python code and is extremely dangerous.

Never use exec() with untrusted input. This can lead to arbitrary code execution.
Consider refactoring to use proper functions, classes, or configuration files instead."#.to_string(),
        },

        // ===== SQL =====
        SecurityPattern {
            rule_name: "sql_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["execute(\"".to_string(), "execute('".to_string(), "executemany(\"".to_string(), "executemany('".to_string()],
            reminder: r#"⚠️ Security Warning: String interpolation in SQL queries can lead to SQL injection vulnerabilities.

Instead of:
  cursor.execute(f"SELECT * FROM users WHERE name = '{user_input}'")

Use parameterized queries:
  cursor.execute("SELECT * FROM users WHERE name = ?", (user_input,))

Always use parameter substitution for user-controlled values."#.to_string(),
        },
        SecurityPattern {
            rule_name: "sql_string_format".to_string(),
            path_pattern: None,
            content_substrings: vec!["query(format!".to_string(), "execute(format!".to_string(), "query(&format!".to_string()],
            reminder: r#"⚠️ Security Warning: String formatting in SQL queries can lead to SQL injection.

Instead of:
  conn.execute(&format!("SELECT * FROM users WHERE name = '{}'", name))

Use parameterized queries:
  conn.execute("SELECT * FROM users WHERE name = ?1", [name])

Always use parameter binding for dynamic values."#.to_string(),
        },

        // ===== Rust =====
        SecurityPattern {
            rule_name: "rust_unsafe_block".to_string(),
            path_pattern: None,
            content_substrings: vec!["unsafe {".to_string(), "unsafe{".to_string()],
            reminder: r#"⚠️ Security Warning: Unsafe blocks bypass Rust's safety guarantees.

Unsafe code can lead to:
- Memory unsafety (use-after-free, buffer overflows)
- Data races
- Undefined behavior
- Null pointer dereferencing

Document why unsafe is necessary and ensure all invariants are upheld.
Consider using safe abstractions instead."#.to_string(),
        },
        SecurityPattern {
            rule_name: "rust_command_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["Command::new(\"/bin/sh\")".to_string(), "Command::new(\"sh\")".to_string(), "Command::new(\"bash\")".to_string()],
            reminder: r#"⚠️ Security Warning: Using shell commands can lead to command injection vulnerabilities.

Instead of:
  Command::new("sh").arg("-c").arg(format!("cmd {}", user_input))

Use direct command execution:
  Command::new("cmd").arg(user_input)

Avoid shells unless absolutely necessary. Never interpolate user input into shell commands."#.to_string(),
        },

        // ===== Go =====
        SecurityPattern {
            rule_name: "go_command_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["exec.Command(\"sh\"".to_string(), "exec.Command(\"bash\"".to_string(), "exec.Command(\"/bin/sh\"".to_string()],
            reminder: r#"⚠️ Security Warning: Using shell commands can lead to command injection.

Instead of:
  cmd := exec.Command("sh", "-c", fmt.Sprintf("cmd %s", userInput))

Use direct command execution:
  cmd := exec.Command("cmd", userInput)

Avoid shells unless absolutely necessary."#.to_string(),
        },
        SecurityPattern {
            rule_name: "go_sql_injection".to_string(),
            path_pattern: None,
            content_substrings: vec!["db.Exec(fmt.Sprintf".to_string(), "db.Query(fmt.Sprintf".to_string(), "db.QueryRow(fmt.Sprintf".to_string()],
            reminder: r#"⚠️ Security Warning: String formatting in SQL queries leads to SQL injection.

Instead of:
  db.Query(fmt.Sprintf("SELECT * FROM users WHERE id = %d", userID))

Use parameterized queries:
  db.Query("SELECT * FROM users WHERE id = ?", userID)

Always use placeholders (?) for dynamic values."#.to_string(),
        },

        // ===== Swift =====
        SecurityPattern {
            rule_name: "swift_force_unwrap".to_string(),
            path_pattern: None,
            content_substrings: vec!["!".to_string()],
            reminder: r#"⚠️ Security Warning: Force unwrapping (!) can cause runtime crashes if the value is nil.

Instead of:
  let value = optional!

Use optional binding or nil coalescing:
  guard let value = optional else { return }
  // or
  let value = optional ?? defaultValue

Only force unwrap when you have absolute certainty the value exists."#.to_string(),
        },
        SecurityPattern {
            rule_name: "swift_unsafe_operations".to_string(),
            path_pattern: None,
            content_substrings: vec!["unsafeBitCast".to_string(), "UnsafeMutablePointer".to_string(), "UnsafeRawPointer".to_string()],
            reminder: r#"⚠️ Security Warning: Unsafe pointer operations bypass Swift's memory safety guarantees.

Unsafe operations can lead to:
- Memory corruption
- Use-after-free vulnerabilities
- Data races
- Undefined behavior

Only use unsafe operations when absolutely necessary and document why they're required."#.to_string(),
        },
        SecurityPattern {
            rule_name: "swift_nspredicate_format".to_string(),
            path_pattern: None,
            content_substrings: vec!["NSPredicate(format:".to_string()],
            reminder: r#"⚠️ Security Warning: NSPredicate with format strings can be vulnerable to injection attacks.

Instead of:
  NSPredicate(format: "name == '\(userInput)'")

Use parameterized predicates:
  NSPredicate(format: "name == %@", userInput)

Always use parameter substitution (%@) for dynamic values."#.to_string(),
        },

        // ===== Java =====
        SecurityPattern {
            rule_name: "java_runtime_exec".to_string(),
            path_pattern: None,
            content_substrings: vec!["Runtime.getRuntime().exec".to_string()],
            reminder: r#"⚠️ Security Warning: Runtime.exec() can lead to command injection vulnerabilities.

Instead of:
  Runtime.getRuntime().exec("cmd " + userInput)

Use ProcessBuilder with separated arguments:
  new ProcessBuilder("cmd", userInput).start()

Never concatenate user input into shell commands."#.to_string(),
        },
        SecurityPattern {
            rule_name: "java_deserialization".to_string(),
            path_pattern: None,
            content_substrings: vec!["ObjectInputStream".to_string(), "readObject()".to_string()],
            reminder: r#"⚠️ Security Warning: Deserializing untrusted data can lead to remote code execution.

ObjectInputStream deserialization vulnerabilities:
- Can execute arbitrary code
- Exploitable through gadget chains
- Common source of critical CVEs

Use safer alternatives like JSON or validate with look-ahead deserialization."#.to_string(),
        },

        // ===== PHP =====
        SecurityPattern {
            rule_name: "php_eval".to_string(),
            path_pattern: None,
            content_substrings: vec!["eval(".to_string()],
            reminder: r#"⚠️ Security Warning: eval() executes arbitrary PHP code and is extremely dangerous.

Never use eval() with user input. Consider alternatives:
- For variable variables: Use arrays instead
- For dynamic code: Refactor to use functions/methods
- For configuration: Use JSON or YAML parsing

eval() is almost never necessary and creates severe security risks."#.to_string(),
        },
        SecurityPattern {
            rule_name: "php_unserialize".to_string(),
            path_pattern: None,
            content_substrings: vec!["unserialize(".to_string()],
            reminder: r#"⚠️ Security Warning: unserialize() with untrusted data can lead to object injection attacks.

Instead of:
  $data = unserialize($_POST['data'])

Use JSON:
  $data = json_decode($_POST['data'], true)

Only unserialize from trusted sources. Consider using PHP's session handlers instead."#.to_string(),
        },

        // ===== Ruby =====
        SecurityPattern {
            rule_name: "ruby_eval".to_string(),
            path_pattern: None,
            content_substrings: vec!["eval(".to_string(), "instance_eval(".to_string(), "class_eval(".to_string()],
            reminder: r#"⚠️ Security Warning: eval() executes arbitrary Ruby code and is dangerous.

Never use eval() with user input. Alternatives:
- Use send() with whitelisted method names
- Use case/when statements
- Refactor to use proper OOP patterns

eval() creates severe security vulnerabilities."#.to_string(),
        },
        SecurityPattern {
            rule_name: "ruby_yaml_load".to_string(),
            path_pattern: None,
            content_substrings: vec!["YAML.load(".to_string()],
            reminder: r#"⚠️ Security Warning: YAML.load can execute arbitrary Ruby code from untrusted input.

Instead of:
  YAML.load(user_input)

Use safe_load:
  YAML.safe_load(user_input)

YAML.load with untrusted data is equivalent to eval() and can lead to RCE."#.to_string(),
        },
    ]
}
