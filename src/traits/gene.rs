pub trait GeneT: Default + Clone + Sync + Send {
    fn new() -> Self{
        Default::default()
    }
    fn default(mut self) -> Self{
        self.set_id(-1);
        self
    }
    fn get_id(&self) -> i32{0}
    fn set_id(&mut self, id: i32) -> &mut Self;
}