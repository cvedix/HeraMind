use heramind_agent::prompts::builder::PromptBuilder;

fn main() {
    let builder = PromptBuilder::new();
    let prompt = builder.build_system_prompt();

    println!("=== First 800 characters of the prompt ===");
    println!("{}", &prompt[0..800.min(prompt.len())]);

    println!("\n\n=== Checking for language policy ===");
    println!(
        "Contains 'Highest Priority': {}",
        prompt.contains("Highest Priority")
    );
    println!(
        "Contains 'EXACT SAME LANGUAGE': {}",
        prompt.contains("EXACT SAME LANGUAGE")
    );
}
