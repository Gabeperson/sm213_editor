use egui::{
    text::{LayoutJob, LayoutSection},
    Color32, RichText, Stroke, TextFormat,
};
use logos::Logos;
use oneshot::{channel, Sender};
use serde::{Deserialize, Serialize};
use sm213_parser::ParseError;
use std::{fmt::Write, ops::Range};

#[derive(Serialize, Deserialize)]
pub struct App {
    code: String,
    monospace_size: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            code: String::new(),
            monospace_size: 10.,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self::default()
    }
}

fn highlight_inner(code: &str, font_id: &egui::FontId, send: Sender<ParseError>) -> LayoutJob {
    let reg = TextFormat::simple(font_id.clone(), Color32::LIGHT_BLUE);
    let instruction = TextFormat::simple(font_id.clone(), Color32::from_rgb(0x60, 0xa0, 0xd0));
    let paren = TextFormat::simple(font_id.clone(), Color32::from_rgb(0xfc, 0xd3, 0x03));
    let white = TextFormat::simple(font_id.clone(), Color32::WHITE);
    let number = TextFormat::simple(font_id.clone(), Color32::from_rgb(13, 170, 2));
    let comment = TextFormat::simple(font_id.clone(), Color32::from_rgb(0x50, 0x80, 0x50));
    let label = TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 230, 110));
    let mut job = LayoutJob::default();
    let mut lexer = Token::lexer(code);
    while let Some(token) = lexer.next() {
        let format = match token {
            Ok(token) => match token {
                Token::OpenParen | Token::CloseParen => paren.clone(),
                Token::Reg => reg.clone(),
                Token::HexNumber | Token::DecNumber => number.clone(),
                Token::Dollar => number.clone(),
                Token::Comma => white.clone(),
                Token::Comment => comment.clone(),
                Token::DoubleIndirect => paren.clone(),
                Token::Label | Token::Colon => label.clone(),
                // instructions like ld, st, etc.
                _ => instruction.clone(),
            },
            Err(_) => white.clone(),
        };
        job.append(lexer.slice(), 0.0, format)
    }
    job.append(lexer.remainder(), 0.0, white.clone());
    let parsed = sm213_parser::parse(code);
    match parsed {
        Ok(_program) => {
            // Nothing wrong with the program. We don't do anything special.
        }
        Err(e) => {
            let span: Range<usize> = match e.span_or_pos {
                sm213_parser::SpanOrPos::Span(s) => s.into(),
                sm213_parser::SpanOrPos::Pos(s) => s..s + 1,
            };
            parse_highlight(&mut job.sections, span);
            send.send(e).unwrap();
        }
    }
    job
}

fn parse_highlight(v: &mut Vec<LayoutSection>, r: Range<usize>) {
    let mut i = 0;
    let stroke = Stroke::new(3., Color32::from_rgb(255, 0, 0));
    while v.len() > i {
        let (v_start, v_end) = (v[i].byte_range.start, v[i].byte_range.end);
        let (r_start, r_end) = (r.start, r.end);
        if v_end <= r.start || r.end <= v_start {
            i += 1;
            continue;
        }
        match (r_start.cmp(&v_start), r_end.cmp(&v_end)) {
            (std::cmp::Ordering::Less, std::cmp::Ordering::Less)
            | (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => {
                let new = r_end..v_end;
                v[i].byte_range.end = r_end;
                i += 1;
                let mut new_section = v[i].clone();
                new_section.byte_range = new;
                v.insert(i, new_section);
                v[i].format.underline = stroke;
            }
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal)
            | (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
                let new = r_start..v_end;
                v[i].byte_range.end = r_start;
                i += 1;
                let mut new_section = v[i].clone();
                new_section.format.underline = stroke;
                new_section.byte_range = new;
                v.insert(i, new_section);
            }
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => {
                let new1 = r_start..r_end;
                let new2 = r_end..v_end;
                v[i].byte_range.end = r_start;
                i += 1;
                let mut new_section = v[i].clone();
                let mut new_section2 = v[i].clone();
                new_section.format.underline = stroke;
                new_section.byte_range = new1;
                v.insert(i, new_section);
                i += 1;
                new_section2.format.underline = stroke;
                new_section2.byte_range = new2;
                v.insert(i, new_section2);
            }
            _ => v[i].format.underline = stroke,
        }
        i += 1;
    }
}

#[derive(Default)]
struct Highlighter;
// Taken from egui_extras::syntax_highlighting::highlight and modified
fn highlight(
    ctx: &egui::Context,
    style: &egui::Style,
    code: &str,
) -> (LayoutJob, Option<ParseError>) {
    impl egui::util::cache::ComputerMut<(&str, &egui::FontId), (LayoutJob, Option<ParseError>)>
        for Highlighter
    {
        fn compute(
            &mut self,
            (code, font_id): (&str, &egui::FontId),
        ) -> (LayoutJob, Option<ParseError>) {
            let (send, recv) = channel();
            let job = highlight_inner(code, font_id, send);
            match recv.try_recv() {
                Ok(e) => (job, Some(e)),
                Err(e) => match e {
                    oneshot::TryRecvError::Empty => unreachable!(),
                    oneshot::TryRecvError::Disconnected => (job, None),
                },
            }
        }
    }
    type HighlightCache =
        egui::util::cache::FrameCache<(LayoutJob, Option<ParseError>), Highlighter>;
    let font_id = style
        .override_font_id
        .clone()
        .unwrap_or_else(|| egui::TextStyle::Monospace.resolve(style));
    ctx.memory_mut(|mem| mem.caches.cache::<HighlightCache>().get((code, &font_id)))
}
#[derive(Default)]
struct LineNumberer;
// Taken from egui_extras::syntax_highlighting::highlight and modified
fn line_numberer(ctx: &egui::Context, style: &egui::Style, code: &str) -> String {
    impl egui::util::cache::ComputerMut<(&str, &egui::FontId), String> for LineNumberer {
        fn compute(&mut self, (code, _font_id): (&str, &egui::FontId)) -> String {
            let mut buffer = String::new();
            // we use split because for .lines(), the last newline is optional
            for (i, _l) in code.split("\n").enumerate() {
                writeln!(buffer, "{: >5}", i + 1).unwrap();
            }
            // remove last newline for alignment
            buffer.pop();
            buffer
        }
    }
    type HighlightCache = egui::util::cache::FrameCache<String, LineNumberer>;
    let font_id = style
        .override_font_id
        .clone()
        .unwrap_or_else(|| egui::TextStyle::Monospace.resolve(style));
    ctx.memory_mut(|mem| mem.caches.cache::<HighlightCache>().get((code, &font_id)))
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Increase font size").clicked() {
                    self.monospace_size += 1.;
                }
                if ui.button("Decrease font size").clicked() {
                    self.monospace_size -= 1.;
                }
            });
        });

        self.monospace_size = self.monospace_size.clamp(10., 23.);
        ctx.style_mut(|s| {
            if let Some(fontid) = s.text_styles.get_mut(&egui::TextStyle::Monospace) {
                fontid.size = self.monospace_size;
            }
        });
        let mut err = None;
        egui::SidePanel::left("code_panel")
            .min_width(ctx.input(|i: &egui::InputState| i.screen_rect().width() * (1. / 2.)))
            .max_width(ctx.input(|i: &egui::InputState| i.screen_rect().width() - 200.))
            .show(ctx, |ui| {
                let mut layouter = |ui: &egui::Ui, code: &str, wrap_width: f32| {
                    let (mut layout_job, e) = highlight(ctx, ui.style(), code);
                    err = e;
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut line_numbers = line_numberer(ctx, ui.style(), &self.code);
                        let desired_rows = 60;
                        ui.add_enabled(
                            false,
                            egui::TextEdit::multiline(&mut line_numbers)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false)
                                .desired_rows(desired_rows)
                                .horizontal_align(egui::Align::Max)
                                .desired_width(70.),
                        );
                        ui.add(
                            egui::TextEdit::multiline(&mut self.code)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(desired_rows)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        );
                    });
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(e) = err {
                let message = e.message.to_string();
                let start = match e.span_or_pos {
                    sm213_parser::SpanOrPos::Span(s) => s.start,
                    sm213_parser::SpanOrPos::Pos(p) => p,
                };
                let (line, column) = find_line_and_column(&self.code, start);
                let err_msg = format!("Error at {line}:{column}:\n{message}");
                // let err_msg = match find_line_and_column(&self.code, start) {
                //     Some((line, column)) => format!("Error at {line}:{column}:\n{message}"),
                //     None => format!("Error occured: {message}"),
                // };
                ui.label(
                    RichText::new(err_msg)
                        .size(self.monospace_size * 1.5)
                        .color(Color32::LIGHT_RED),
                );
            }
        });
    }
}

fn find_line_and_column(text: &str, offset: usize) -> (usize, usize) {
    let mut current_offset = 0;

    for (line_num, line) in text.lines().enumerate() {
        let line_length = line.len() + 1;
        if current_offset + line_length > offset {
            let column = offset - current_offset;
            return (line_num + 1, column + 1);
        }

        current_offset += line_length;
    }

    // Should only recurse once, at maximum
    let (line, _column) = find_line_and_column(text, offset - 1);
    (line + 1, 1)
}

#[derive(Logos, Debug, Clone)]
// #[logos(skip r#"[\t \n]+"#)]
pub enum Token {
    #[regex(r#"r\d+"#)]
    Reg,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[regex(r#"0x[a-fA-F\d]+"#)]
    HexNumber,
    #[regex(r#"-?[\d]+"#)]
    DecNumber,
    #[token("$")]
    Dollar,
    #[token(",")]
    Comma,
    #[regex(r#"#.*"#)]
    Comment,
    #[token("*")]
    DoubleIndirect,
    #[regex(r#"[a-zA-Z_][a-zA-Z_\d]*"#)]
    Label,
    #[token(":")]
    Colon,
    #[regex(r"ld\s")]
    Load,
    #[regex(r"st\s")]
    Store,
    #[regex(r#"halt\s"#)]
    Halt,
    #[regex(r#"nop\s"#)]
    Nop,
    #[regex(r"mov\s")]
    Mov,
    #[regex(r"add\s")]
    Add,
    #[regex(r"and\s")]
    And,
    #[regex(r"inc\s")]
    Inc,
    #[regex(r"inca\s")]
    Inca,
    #[regex(r"dec\s")]
    Dec,
    #[regex(r"deca\s")]
    Deca,
    #[regex(r"not\s")]
    Not,
    #[regex(r"shl\s")]
    Shl,
    #[regex(r"shr\s")]
    Shr,
    #[regex(r"br\s")]
    Branch,
    #[regex(r"bgt\s")]
    BranchIfGreater,
    #[regex(r"beq\s")]
    BranchIfEqual,
    #[regex(r"j\s")]
    Jump,
    #[regex(r"gpc\s")]
    GetProgramCounter,
    #[regex(r"sys\s")]
    Syscall,
    #[regex(r#"\.pos\s"#)]
    PosDirective,
    #[regex(r#"\.long\s"#)]
    LongDirective,
}
