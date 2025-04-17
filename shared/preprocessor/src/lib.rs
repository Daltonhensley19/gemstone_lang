use std::collections::BTreeSet;

use module_manager::{Module, ModuleManager};
use span::Span;

pub struct Preprocessor<'preprocessor> {
    pub module_manager: &'preprocessor mut ModuleManager,
    cursor: usize,
    span: Span,
}

/// CTOR
impl<'preprocessor> Preprocessor<'preprocessor> {
    pub fn new(module_manager: &'preprocessor mut ModuleManager) -> Self {
        let span = Span::new();
        Self {
            module_manager,
            cursor: 0,
            span,
        }
    }
}

impl<'preprocessor> Preprocessor<'preprocessor> {
    pub fn detect_nonvalid_chars(&self) {
        let modules = self.module_manager.get_ref();

        let mut detect_nonvalid_chars = |module: &Module| {
            let alpha_lower = vec![
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            ];

            let alpha_upper = vec![
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ];

            let numeric = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

            let whitespace = vec![' ', '\n', '\t'];

            let punc = vec![';', '(', ')', '[', ']', '{', '}', ','];

            let ops = vec![
                '+', '-', '*', '/', '%', '<', '>', '!', '?', '&', '|', '^', '~', '=', ':', '.',
            ];

            let white_list = vec![alpha_lower, alpha_upper, numeric, punc, whitespace, ops]
                .into_iter()
                .flat_map(|v| v)
                .collect::<Vec<char>>();

            let mut error_span = Span::new();
            let invalid_char_found = modules.iter().any(|m| {
                let module_chrs = m.src.chars();

                module_chrs.enumerate().any(|(_, c)| {
                    error_span.incre_from_char(c);
                    !white_list.contains(&c)
                })
            });

            if invalid_char_found {
                panic!(
                    "Found invalid character in module: {}\t{}:{}",
                    module.path.display(),
                    error_span.line_num,
                    error_span.col_num
                );
            }
        };

        // Non-mutable preprocessing
        for module in modules {
            detect_nonvalid_chars(module);
        }
    }

    pub fn strip_comments(&mut self) {
        let mut modules = self.module_manager.get_mut_ref();

        let mut strip_comments_for = |module: &mut Module| {
            let src_len = module.src.len();

            // While not at EOF
            while self.cursor < src_len {
                let Some(current_chr) = module.src.chars().nth(self.cursor) else {
                    return;
                };

                // If we are at a blank newline
                if current_chr == '\n' {
                    self.cursor += 1;
                    self.span.incre_line_num();
                    self.span.reset_col_num();
                    continue;
                }

                // If we are at a tab
                if current_chr == '\t' {
                    module.src.remove(self.cursor);
                    self.cursor += 4;
                    self.span.incre_col_num_by(4);

                    continue;
                }

                // Remove single newline comment
                if current_chr == '/' {
                    let next_chr = module.src.chars().nth(self.cursor + 1);
                    if next_chr == Some('/') {
                        module.src.remove(self.cursor);
                        module.src.remove(self.cursor);

                        let mut next_chr = module.src.chars().nth(self.cursor);
                        while next_chr != Some('\n') {
                            module.src.remove(self.cursor);
                            next_chr = module.src.chars().nth(self.cursor);
                        }

                        // At a newline here!
                        module.src.remove(self.cursor);
                    }
                }
                self.cursor += 1;
            }
        };

        // Mutable preprocessing
        for module in modules {
            strip_comments_for(module);
        }
    }
}
