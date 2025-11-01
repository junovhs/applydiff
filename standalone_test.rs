// Standalone test for line extraction
// Run with: rustc standalone_test.rs && ./standalone_test

fn main() {
    let content = "1\n2\n3\n4\n5";
    let range_spec = "2-4";
    
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    
    println!("Content: {:?}", content);
    println!("Lines vector: {:?}", lines);
    println!("Total: {}", total);
    println!();
    
    // Parse range
    let (start_line, end_line) = if let Some((s, e)) = range_spec.split_once('-') {
        (s.trim().parse::<usize>().unwrap(), e.trim().parse::<usize>().unwrap())
    } else {
        panic!("Expected range");
    };
    
    println!("Parsed: start_line={}, end_line={}", start_line, end_line);
    
    // Convert to indices
    let start_idx = start_line.saturating_sub(1);
    let end_idx = end_line;
    
    println!("Indices: start_idx={}, end_idx={}", start_idx, end_idx);
    
    // Extract
    let slice = lines.get(start_idx..end_idx).unwrap_or(&[]);
    println!("Slice: {:?}", slice);
    
    let extracted = slice.join("\n");
    println!("Extracted: {:?}", extracted);
    
    // Build markdown like the real code does
    let info = format!("lines {}-{} of {}", start_line, end_line, total);
    let markdown = format!("## file.txt\n*Showing: {}*\n```\n{}\n```\n", info, extracted);
    
    println!("\nMarkdown:");
    println!("{}", markdown);
    
    println!("\nChecks:");
    println!("  contains('1'): {}", markdown.contains('1'));
    println!("  contains(\"2\\n3\\n4\"): {}", markdown.contains("2\n3\n4"));
    println!("  contains(\"lines 2-4 of 5\"): {}", markdown.contains("lines 2-4 of 5"));
    
    // Test assertion
    if markdown.contains("lines 2-4 of 5") && markdown.contains("2\n3\n4") && !markdown.contains('1') {
        println!("\n✅ TEST PASSED");
    } else {
        println!("\n❌ TEST FAILED");
        println!("  Expected: contains(\"lines 2-4 of 5\") AND contains(\"2\\n3\\n4\") AND NOT contains('1')");
    }
}