use static_files::resource_dir;

fn main() {
    resource_dir("./gui/build")
        .build()
        .expect("loading resource failed");
}
