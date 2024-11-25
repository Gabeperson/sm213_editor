<h1 align="center">SM213 Editor</h1>

<p align="center">
  <img alt="badge-lastcommit" src="https://img.shields.io/github/last-commit/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-openissues" src="https://img.shields.io/github/issues-raw/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-license" src="https://img.shields.io/github/license/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-contributors" src="https://img.shields.io/github/contributors/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-codesize" src="https://img.shields.io/github/languages/code-size/Gabeperson/sm213_editor?style=for-the-badge">
</p>

sm213_editor is a web-based code editor for the SM213 language. It is hosted [here](https://gabeperson.github.io/sm213_editor/).

## Features
- **Live error reporting:** Errors are displayed in real time. Error messages are descriptive so that you can understand what's wrong.
- **Browser-based:** You don't need to install anything to run. Just go to [https://gabeperson.github.io/sm213_editor/](https://gabeperson.github.io/sm213_editor/) and start writing.
- **Light mode and Dark mode:** Pretty self explanatory.
- **Code Formatting (Experimental):** Reformats your code in a nice-to-view way.

## Using the code editor
Goto [https://gabeperson.github.io/sm213_editor/](https://gabeperson.github.io/sm213_editor/).

## Development
For local development, you have to do the following:
1. Install [Rust](https://www.rust-lang.org/).
2. Install [Node](https://nodejs.org/en).
3. Install [Just](https://github.com/casey/just).
4. Install [Wasm-pack](https://github.com/rustwasm/wasm-pack).
5. From the root directory, run `just watchwasm` in one terminal and `just watchvite` in another.
6. Navigate to `localhost:5173`.

## Architecture
The frontend, located in the `sm213_editor` subdirectory in the repo, is made in HTML, CSS, and Typescript with no frameworks. It leverages [Monaco Editor](https://github.com/microsoft/monaco-editor) for the editor ui.

The [parser](https://github.com/gabeperson/sm213_parser) is made in rust with a [custom parser combinator framework](https://github.com/Gabeperson/parser) designed for returning great errors. (I might implement a ground up parser in the future for even better errors)

The frontend communicates with the backend (compiled to [wasm](https://webassembly.org/)) using a lightweight js <-> wasm bridge, located in the `sm213_parser_wasm` subdirectory in this repo.

## Future Goals (approximately in order)
- Instruction overwrite & unaligned instruction detection
- "Run" button in editor to actually run the code directly without having to copy it over
- Debugging support

## Contributing
Contributions are welcome!

Please report any issues or feature requests using GitHub issues on this repository.

If there is an issue that you would like to work on, feel free to leave a comment in the corresponding GitHub issue and then make a PR.

## License
This work is licensed under the [MIT License](https://github.com/Gabeperson/sm213_editor/blob/main/LICENSE-MIT).
