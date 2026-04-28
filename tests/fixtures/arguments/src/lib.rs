use methodify::methodify;

#[methodify(Executable)]
fn is_executable(path: &std::path::Path) -> bool {
    path.is_file()
}
