
struct Dropper<T> {
    
}

impl<T> Dropper<T> {
    
    pub fn drop(t:T){}
}

trait DropperLike {
    type I;
    pub fn drop(i: I) {

    }
}
impl<T> DropperLike for Dropper<T> {
    type I = T;
    fn drop(i: I) {
        
    }
}



fn main() {
    let v: std::vec::Vec<impl DropperLike> = std::vec::Vec::new();
}
