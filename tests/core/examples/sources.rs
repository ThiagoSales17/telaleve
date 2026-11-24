fn main() {
    match telaleve_core_tests::capture::sources() {
        Ok(s) => println!("{} origens X11 disponíveis", s.len()),
        Err(e) => {
            eprintln!("X11 indisponível: {e}");
            std::process::exit(1)
        }
    }
}
