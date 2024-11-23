<h1 align="center">SM213 editor</h1>

<p align="center">
  <img alt="badge-lastcommit" src="https://img.shields.io/github/last-commit/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-openissues" src="https://img.shields.io/github/issues-raw/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-license" src="https://img.shields.io/github/license/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-contributors" src="https://img.shields.io/github/contributors/Gabeperson/sm213_editor?style=for-the-badge">
  <img alt="badge-codesize" src="https://img.shields.io/github/languages/code-size/Gabeperson/sm213_editor?style=for-the-badge">
</p>

sm213_editor is a web-based code editor for the SM213 language. It is hosted [here](https://gabeperson.github.io/sm213_editor/). It provides live syntactic error reporting as you code, powered by [sm213_parser](https://github.com/gabeperson/sm213_parser).

## Features
- **Live error reporting:** Syntax errors are displayed in real time. Error messages are descritive so that you can find the error easily.
- **Browser-based:** You don't need to install anything to run. Just go to [https://gabeperson.github.io/sm213_editor/](https://gabeperson.github.io/sm213_editor/) and start writing.
- **Different color modes:** Currently supports light and dark mode.

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

## Contributing
Please report any issues or feature requests using GitHub issues on this repository. If there is an issue that you would like to work on, feel free to leave a comment in the corresponding GitHub issue and then make a PR.
## License
This work is licensed under the [MIT license](https://github.com/Gabeperson/sm213_editor/LICENSE-MIT).
