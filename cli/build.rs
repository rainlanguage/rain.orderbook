use static_files::resource_dir;

fn main() {
    resource_dir("./gui/dist")
        .build()
        .expect("loading resource failed");
}
