fn main() {
    if let Err(e) = samsara::run() {
        eprintln!("{}: {e}", samsara::i18n::t("error"));
        std::process::exit(1);
    }
}
