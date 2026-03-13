/// Embeds dashboard files at compile time into a single HTML string.
pub fn render() -> String {
    let html = include_str!("index.html");
    let css = include_str!("style.css");
    let js = include_str!("script.js");
 
    html.replace("/* __CSS__ */", css)
        .replace("/* __JS__ */", js)
}
 
