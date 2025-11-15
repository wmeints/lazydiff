# Solution Strategy

## Technology choices

- We use Rust to make the application blazingly fast in startup and use.
- We use `ratatui` to build the terminal interface so we can render widgets, etc.
- We use `arboard` to copy data to the clipboard as this package support xplatform.
- We use `clap` to build the CLI interface.

## Development process

- We'll use Claude Code to build the application with manual reviews and 
  automated CI/CD workflows to validate the code before publishing it.

- We'll write the code out in the public space with an MIT license so others
  can contribute to the application.
