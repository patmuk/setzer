setzer::site!(); // the pages in src/www/ → module `www`

fn main() -> setzer::Result {
    setzer::Site::new("https://example.com", "Hello", &[setzer::Lang::En]).write()
}
