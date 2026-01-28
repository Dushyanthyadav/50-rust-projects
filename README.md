# 50 Rust Projects 🦀

A personal journey to master the Rust programming language by building 50 projects, ranging from simple scripts to complex systems utilities.

![Progress](https://img.shields.io/badge/Progress-9%2F50-orange) 
## 📂 Project Structure

This repository is organized as a **Cargo Workspace**. All projects share a single `target` directory to save disk space and compilation time.

### How to Run a Project
Since this is a workspace, you must specify which package to run using the `-p` flag:

```bash
# General syntax
cargo run -p <project-name>

# Example
cargo run -p guessing-game-02