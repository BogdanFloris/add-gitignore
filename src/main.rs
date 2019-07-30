extern crate console;
extern crate dialoguer;

mod client;
mod term;

use dialoguer::theme::ColorfulTheme;
use std::fs::File;
use std::io::Write;
use term::Terminal;

fn main() -> std::io::Result<()> {
    // Call the API to get technologies
    let technologies = client::get_technologies().unwrap();

    // Get the checkboxes for the CLI
    let checkboxes: Vec<&str> = technologies.iter().map(AsRef::as_ref).collect();

    // Open the interactive terminal and let the user select the technologies
    let selections = Terminal::new(&ColorfulTheme::default(), 10)
        .items(&checkboxes)
        .interact()
        .unwrap();

    // The user did not select anything
    if selections.is_empty() {
        println!("You did not select anything :(");
    } else {
        println!("Generated `.gitignore`'s for the following technologies:");
        // The user selected one technology, generate `.gitignore`
        if selections.len() == 1 {
            let mut file = File::create(".gitignore")?;
            file.write_all(
                client::get_gitignore(checkboxes[selections[0]])
                    .unwrap()
                    .as_bytes(),
            )?;
            println!("  {}", checkboxes[selections[0]]);
        // The user selected multiple technologies,
        // generate `.gitignore_{tech}` for each technology
        } else {
            for selection in selections {
                let mut file = File::create(
                    format!(".gitignore_{}", checkboxes[selection])
                )?;
                file.write_all(
                    client::get_gitignore(checkboxes[selection])
                        .unwrap()
                        .as_bytes(),
                )?;
                println!("  {}", checkboxes[selection]);
            }
        }
    }

    Ok(())
}
