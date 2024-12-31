use crate::traits::GeneT;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Binary{
    pub id: i32,
    pub value: bool,
}
impl GeneT for Binary {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

impl Binary{
    pub fn new(&mut self, id: i32, value: bool) -> &mut Self {
        self.id = id;
        self.value = value;
        self
    }
    fn get_value(&self) -> bool {
        self.value
    }
    fn set_value(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }
}