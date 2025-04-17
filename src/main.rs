use lexical_analyzer::Scanner;
use module_manager::ModuleManager;
use parser;
use preprocessor::Preprocessor;

use log;
use pretty_env_logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register logging system
    pretty_env_logger::init();

    // Get modules
    let mut module_manager = ModuleManager::new()?;

    // Preprocess modules
    let mut preprocessor = Preprocessor::new(&mut module_manager);
    preprocessor.detect_nonvalid_chars();
    preprocessor.strip_comments();

    // Get the token stream for each module
    let scanner = Scanner::new(&mut module_manager);
    let scanner_data_from_modules = scanner.scan()?;

    for module_scanner_data in &scanner_data_from_modules {
        let tokens = &module_scanner_data.tokens;
        log::info!(
            "Tokens for module: {}",
            module_scanner_data.module.path.display()
        );

        log::info!("{tokens:#?} tokens");
    }

    let ast = parser::Ast::new(scanner_data_from_modules);

    Ok(())
}
