use zed_extension_api as zed;

#[derive(Default)]
struct KiraExtension;

impl zed::Extension for KiraExtension {
    fn new() -> Self {
        Self
    }
}

zed::register_extension!(KiraExtension);
