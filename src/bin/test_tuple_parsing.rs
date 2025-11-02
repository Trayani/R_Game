/// Test tuple parsing in expressions

use rustgame3::dsl::expression_parser::ExpressionParser;

fn main() {
    let test_cases = vec![
        ("(1, 2)", "2-element int tuple"),
        ("(x, y)", "2-element variable tuple"),
        ("(V, south, false)", "3-element mixed tuple"),
        ("(a + b, c * d)", "tuple with expressions"),
        ("(func(), x, 123)", "tuple with function call"),
        ("((a, b), c)", "nested tuple"),
        ("()", "empty tuple"),
        ("(1,)", "single element with trailing comma"),
        ("(a)", "grouped expression, not tuple"),
    ];

    println!("🧪 Testing Tuple Parsing\n");

    for (expr_str, description) in test_cases {
        print!("  Testing: {} ... ", description);
        match ExpressionParser::parse_from_str(expr_str) {
            Ok(ast) => {
                println!("✅");
                println!("    Input:  {}", expr_str);
                println!("    Result: {:?}\n", ast);
            }
            Err(e) => {
                println!("❌");
                println!("    Input:  {}", expr_str);
                println!("    Error:  {}\n", e);
            }
        }
    }
}
