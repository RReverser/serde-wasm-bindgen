#[derive(Default)]
pub struct Serializer(());

impl Serializer {
    pub fn new() -> Self {
        Default::default()
    }
}
