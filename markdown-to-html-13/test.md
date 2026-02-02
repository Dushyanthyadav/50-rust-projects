# 🚀 Markdown Stress Test: CommonMark Edition

## 1. Text Formatting & Links
This is a paragraph containing **bold text**, *italicized text*, and ~~strikethrough~~. You can also have **_combined formatting_**. 

Here is a [link to Rust's homepage](https://www.rust-lang.org) and an automatic link: <https://www.rust-lang.org>

---

## 2. Lists (Nested & Task-based)
* **Main Category A**
    * Sub-item 1 with `inline code`.
    * Sub-item 2
        1.  Ordered sub-step
        2.  Ordered sub-step
* [x] Completed task
* [ ] Incomplete task

---

## 3. The Technical Stuff (Code Blocks)

Here is a block of Rust code that uses the `pulldown-cmark` library:

```rust
use pulldown_cmark::{Parser, Options, html};

fn main() {
    let markdown_input = "# Hello world";
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    
    let parser = Parser::new_ext(markdown_input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    println!("{}", html_output);
}