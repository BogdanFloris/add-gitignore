/// Client module
///
/// This module handles all the calling to the: https://www.gitignore.io/ API.
/// This API is used to get the all the technologies for which the
/// `add-gitignore` CLI application can generate `.gitignore` files.
/// When a technology is chosen, we can again call the API
/// to get the actual `.gitignore` file for that chosen technology.
extern crate reqwest;

static API: &str = "https://www.gitignore.io/api/";

/// Returns the technologies for which the application can generate `.gitignore`'s.
pub fn get_technologies() -> Result<Vec<String>, reqwest::Error> {
    // Get the request body
    let body = reqwest::get(&format!("{}{}", API, "list"))?
        .text()?;

    // Transform to a vector of Strings
    let technologies: Vec<String> = body.split(",")
        .map(|s| s.to_string())
        .collect();

    Ok(technologies)
}

/// Returns the `.gitignore` contents given a technology.
pub fn get_gitignore(tech: &str) -> Result<String, reqwest::Error> {
    // Get the request body
    let body = reqwest::get(&format!("{}{}", API, tech))?
        .text()?;

    Ok(body)
}
