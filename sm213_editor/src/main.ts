import * as monaco from "monaco-editor";
// ts-ignores are here because it works, but tsc doesn't seem to be able to
// resolve them strangely.
// @ts-ignore
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
// // @ts-ignore
// import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
// // @ts-ignore
// import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
// // @ts-ignore
// import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
// // @ts-ignore
// import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

// Wasm initialization
import init, * as parserBackend from "./wasm/sm213_parser_wasm.js";
await init();

self.MonacoEnvironment = {
    getWorker(_, _label) {
        // if (label === "json") {
        //     return new jsonWorker();
        // }
        // if (label === "css" || label === "scss" || label === "less") {
        //     return new cssWorker();
        // }
        // if (label === "html" || label === "handlebars" || label === "razor") {
        //     return new htmlWorker();
        // }
        // if (label === "typescript" || label === "javascript") {
        //     return new tsWorker();
        // }
        return new editorWorker();
    },
};

const errorpanel = document.querySelector("#errorpanel")!;

interface ErrorPanelError {
    position: monaco.Position;
    message: string;
    severity: parserBackend.Severity;
    extra_info?: string;
}

monaco.editor.onDidCreateModel((model) => {
    const validate = () => {
        let text = model.getValue();
        localStorage.setItem("code", text);

        let uri = model.uri;

        let result = parserBackend.parse_sm213(text);

        let parsingError = result.parsing_error();
        if (parsingError != undefined) {
            let span = parsingError.span();
            let pos_start = span.start;
            let pos_end = span.end;
            let span_start = model.getPositionAt(pos_start);
            let span_end = model.getPositionAt(pos_end);
            let message = parsingError.message();
            let markers: monaco.editor.IMarkerData[] = [
                {
                    severity: monaco.MarkerSeverity.Error,
                    startLineNumber: span_start.lineNumber,
                    startColumn: span_start.column,
                    endLineNumber: span_end.lineNumber,
                    endColumn: span_end.column,
                    message,
                },
            ];
            monaco.editor.setModelMarkers(model, "sm213", markers);
            setErrorPanelErrors([
                {
                    position: span_start,
                    message,
                    severity: parserBackend.Severity.Error,
                },
            ]);
            return;
        }
        let semanticErrors = result.semantic_errors();

        let diagnostics: monaco.editor.IMarkerData[] = [];

        for (let error of semanticErrors) {
            let severity =
                error.severity() == parserBackend.Severity.Error
                    ? monaco.MarkerSeverity.Error
                    : monaco.MarkerSeverity.Warning;
            let span = error.span();
            let pos_start = span.start;
            let pos_end = span.end;
            let span_start = model.getPositionAt(pos_start);
            let span_end = model.getPositionAt(pos_end);
            let relatedInformation:
                | monaco.editor.IRelatedInformation[]
                | undefined;
            {
                let related = error.related();
                if (related != undefined) {
                    let span = related.span;
                    let pos_start = span.start;
                    let pos_end = span.end;
                    let span_start = model.getPositionAt(pos_start);
                    let span_end = model.getPositionAt(pos_end);
                    let relatedObject: monaco.editor.IRelatedInformation = {
                        startLineNumber: span_start.lineNumber,
                        startColumn: span_start.column,
                        endLineNumber: span_end.lineNumber,
                        endColumn: span_end.column,
                        message: related.message(),
                        resource: uri,
                    };
                    relatedInformation = [relatedObject];
                }
            }
            let marker: monaco.editor.IMarkerData = {
                severity,
                startLineNumber: span_start.lineNumber,
                startColumn: span_start.column,
                endLineNumber: span_end.lineNumber,
                endColumn: span_end.column,
                message: error.message(),
                relatedInformation,
            };
            diagnostics.push(marker);
        }

        monaco.editor.setModelMarkers(model, "sm213", diagnostics);
        let err_panel_errs: ErrorPanelError[] = [];

        for (let diagnostic of diagnostics) {
            let related = diagnostic.relatedInformation;
            let extra =
                related != undefined
                    ? `Related: [program.s (${related[0].startLineNumber}:${related[0].startColumn}): ${related[0].message}]`
                    : undefined;
            err_panel_errs.push({
                position: new monaco.Position(
                    diagnostic.startLineNumber,
                    diagnostic.startColumn,
                ),
                message: diagnostic.message,
                severity:
                    diagnostic.severity == monaco.MarkerSeverity.Error
                        ? parserBackend.Severity.Error
                        : parserBackend.Severity.Warning,
                extra_info: extra,
            });
        }
        setErrorPanelErrors(err_panel_errs);
    };

    let handle: undefined | number;
    model.onDidChangeContent(() => {
        clearTimeout(handle);
        handle = setTimeout(() => validate(), 100);
    });
    validate();
});

monaco.languages.register({ id: "sm213" });

const language_syntax: () => monaco.languages.IMonarchLanguage = () => {
    return {
        defaultToken: "invalid",
        // prettier-ignore
        instruction: [
            "ld", "st", "halt", "nop", "mov", "add",
            "add", "and", "inc", "inca", "dec", "deca",
            "not", "shl", "shr", "br", "bgt", "gpc",
            "beq", "j", "sys",
        ],

        tokenizer: {
            root: [
                [
                    /[^\d\W][\w]*/,
                    {
                        cases: {
                            "r[0-7]": "sm213.register",
                            "@instruction": "sm213.instruction",
                            "@default": "sm213.label",
                        },
                    },
                ],
                [/\.long|\.pos/, "sm213.instruction"],
                [/[ \t]+/, "sm213.whitespace"],
                [/\r\n|\r|\n/, "sm213.newline"],
                [/\(|\)|,|:|\*/, "sm213.punct"],
                [/\$/, "sm213.dollar"],

                [/#.*$/, "sm213.comment"],

                [/0x[0-9a-fA-F]+/, "sm213.hex"],
                [/-?\d+/, "sm213.dec"],
            ],
        },
    };
};

// Register a tokens provider for the language
monaco.languages.setMonarchTokensProvider("sm213", language_syntax());

// Define a new theme that contains only rules that match this language
monaco.editor.defineTheme("sm213-dark", {
    base: "vs-dark",
    inherit: false,
    rules: [
        { token: "sm213.register", foreground: "acd8e6" },
        { token: "sm213.label", foreground: "ffe66e" },
        { token: "sm213.instruction", foreground: "60a0d0" },
        { token: "sm213.punct", foreground: "fcd303" },
        { token: "sm213.hex", foreground: "0daa02" },
        { token: "sm213.comment", foreground: "508050" },
        { token: "sm213.dec", foreground: "0daa02" },
        { token: "sm213.dollar", foreground: "0daa02" },
        { token: "invalid", foreground: "ff0000" },
        { token: "", foreground: "#dddddd" },
    ],
    colors: {},
});

const dark_background = "1e1e1e";

monaco.editor.defineTheme("sm213-light", {
    base: "vs",
    inherit: false,
    rules: [
        { token: "sm213.register", foreground: "f74ff2" },
        { token: "sm213.label", foreground: "509600" },
        { token: "sm213.instruction", foreground: "0000ff" },
        { token: "sm213.punct", foreground: "0daa02" },
        { token: "sm213.hex", foreground: "0daa02" },
        { token: "sm213.comment", foreground: "508050" },
        { token: "sm213.dec", foreground: "0daa02" },
        { token: "sm213.dollar", foreground: "0daa02" },
        { token: "invalid", foreground: "ff0000" },
        { token: "", foreground: "ff7373" },
    ],
    colors: {
        "editor.foreground": "#000000",
    },
});

const light_background = "ffffff";

let model = monaco.editor.createModel(
    localStorage.getItem("code") ?? "# Write your SM213 code here:\n",
    "sm213",
    monaco.Uri.parse("program.s"),
);

let theme: "sm213-dark" | "sm213-light" =
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "sm213-dark"
        : "sm213-light";

let saved_theme: "sm213-dark" | "sm213-light" | null = localStorage.getItem(
    "theme",
) as any;
theme = saved_theme ?? theme;

const right = document.getElementById("right")!;
function setTheme(t: "sm213-dark" | "sm213-light") {
    if (t == "sm213-dark") {
        theme = "sm213-dark";
        right.style.backgroundColor = `#${dark_background}`;
    } else {
        theme = "sm213-light";
        right.style.backgroundColor = `#${light_background}`;
    }
    localStorage.setItem("theme", theme);
    monaco.editor.setTheme(theme);
}
function switchTheme() {
    if (theme == "sm213-dark") {
        setTheme("sm213-light");
    } else {
        setTheme("sm213-dark");
    }
}
setTheme(theme);

(window as any).switchTheme = switchTheme;
(window as any).monaco = monaco;

let editor = monaco.editor.create(document.querySelector("#container")!, {
    theme,
    automaticLayout: true,
    model,
    minimap: {
        enabled: false,
    },
});

function setErrorPanelErrors(errors: ErrorPanelError[]) {
    errorpanel.innerHTML = "";
    for (let err of errors) {
        let err_or_warning =
            err.severity == parserBackend.Severity.Error ? "Error" : "Warning";
        let err_div = document.createElement("div");
        let extra_msg = err.extra_info ? "\n\n" + err.extra_info : "";
        err_div.innerText = `${err_or_warning} at program.s (${err.position.lineNumber}:${err.position.column})\n${err.message}${extra_msg}`;
        err_div.classList.add("errorDiv");
        if (err_or_warning == "Error") {
            err_div.style.color = "#ff6161";
        } else {
            err_div.style.color = "#c9a200";
        }
        err_div.addEventListener("click", () => {
            editor.focus();
            editor.setPosition(err.position);
        });
        errorpanel.appendChild(err_div);
    }
}

function download() {
    // https://stackoverflow.com/questions/3749231/download-file-using-javascript-jquery
    let text = model.getValue();
    const url = window.URL.createObjectURL(
        new Blob([text], { type: "text/plain" }),
    );
    const elem = document.createElement("a");
    elem.style.display = "none";
    elem.href = url;
    elem.download = "program.s";
    document.body.appendChild(elem);
    elem.click();
    document.body.removeChild(elem);
    window.URL.revokeObjectURL(url);
}
(window as any).download = download;

function open_file() {
    // https://stackoverflow.com/questions/16215771/how-to-open-select-file-dialog-via-js

    let input = document.createElement("input");
    input.type = "file";
    input.onchange = (e: any) => {
        let file = e.target.files[0];
        let reader = new FileReader();
        reader.readAsText(file, "UTF-8");
        reader.onload = (readerEvent) => {
            let content = readerEvent.target!.result as string | null;
            if (content != null) {
                model.setValue(content);
            }
        };
    };

    input.click();
}
(window as any).open_file = open_file;

function format() {
    let text = model.getValue();
    let formatted = parserBackend.reformat(text);
    if (formatted == undefined) {
        alert("Program must have no syntax errors to reformat!");
        return;
    }
    editor.pushUndoStop();
    editor.executeEdits("reformat", [
        {
            range: model.getFullModelRange(), // full range
            text: formatted, // target value here
        },
    ]);
    editor.pushUndoStop();
}

(window as any).format = format;
